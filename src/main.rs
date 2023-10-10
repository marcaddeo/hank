use discord::model::Event;
use discord::Discord;
use extism::InternalExt;
use extism::{Function, UserData, Val, ValType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub channel_id: u64,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubscribedEvents(pub HashMap<String, Vec<String>>);

#[allow(dead_code)]
struct Plugin<'a> {
    /// A map of events that this plugin subscribes to along with a list of function names that
    /// should be called for each event.
    pub subscribed_events: SubscribedEvents,

    pub plugin: extism::Plugin<'a>,
}

impl<'a> Plugin<'a> {
    fn new() -> Self {
        let wasm =
            include_bytes!("../plugins/ping/target/wasm32-unknown-unknown/release/ping.wasm");

        let f = Function::new(
            "send_message",
            [ValType::I64],
            [ValType::I64],
            None,
            send_message,
        );
        let mut plugin = extism::Plugin::create(wasm, [f], true).unwrap();
        let output = plugin.call("init", "").unwrap();

        let subscribed_events: SubscribedEvents =
            serde_json::from_str(std::str::from_utf8(output).unwrap()).unwrap();

        Self {
            subscribed_events,
            plugin,
        }
    }
}

struct PluginManager<'a> {
    plugins: Vec<Plugin<'a>>,
}

impl<'a> PluginManager<'a> {
    fn new() -> Self {
        let plugins = vec![Plugin::new()];
        Self { plugins }
    }

    fn dispatch(&mut self, _event: &str, arg: &str) {
        let plugin = self.plugins.get_mut(0).unwrap();

        let _ = plugin.plugin.call("ping_handler", arg);
    }
}

fn discord() -> &'static Arc<Discord> {
    static DISCORD: OnceLock<Arc<Discord>> = OnceLock::new();
    DISCORD.get_or_init(|| {
        Arc::new(Discord::from_bot_token(&env::var("DISCORD_TOKEN").unwrap()).unwrap())
    })
}

fn plugin_manager() -> &'static Mutex<PluginManager<'static>> {
    static PLUGIN_MANAGER: OnceLock<Mutex<PluginManager>> = OnceLock::new();
    PLUGIN_MANAGER.get_or_init(|| Mutex::new(PluginManager::new()))
}

fn send_message(
    plugin: &mut extism::CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    _user_data: UserData,
) -> Result<(), extism::Error> {
    let message: String = plugin
        .memory_read_str(inputs[0].i64().unwrap().try_into().unwrap())
        .unwrap()
        .to_string();
    let message: Message = serde_json::from_str(&message).unwrap();

    let discord = Arc::clone(&discord());
    let _ = discord.send_message(
        discord::model::ChannelId(message.channel_id),
        &message.content,
        "",
        false,
    );

    outputs[0] = inputs[0].clone();
    Ok(())
}

fn main() {
    // Initialize the tracing subscriber.
    tracing_subscriber::fmt::init();

    // Log in to Discord using a bot token from the environment
    let discord =
        Discord::from_bot_token(&env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN missing"))
            .expect("login failed");

    // Establish and use a websocket connection
    let (mut connection, _) = discord.connect().expect("Connect failed");
    info!("Ready.");
    loop {
        match connection.recv_event() {
            Ok(Event::MessageCreate(msg)) => {
                info!("{} says: {}", msg.author.name, msg.content);

                let json = serde_json::to_string(&msg.clone()).unwrap();

                plugin_manager()
                    .lock()
                    .unwrap()
                    .dispatch("MessageCreate", &json);
            }
            Ok(_) => {}
            Err(discord::Error::Closed(code, body)) => {
                error!("Gateway closed on us with code {:?}: {}", code, body);
                break;
            }
            Err(err) => error!("Receive error: {:?}", err),
        }
    }
}
