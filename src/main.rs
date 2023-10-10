use extism::InternalExt;
use extism::{Function, UserData, Val, ValType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::{env, error::Error};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Event, Shard, ShardId};
use twilight_http::Client as HttpClient;
use twilight_model::{gateway::Intents, id::Id};

// use std::any::{Any, TypeId};
// struct Subscriber {
//     event: TypeId,
//     handler: Box<dyn Fn(&mut dyn Any)>,
// }

// struct Dispatcher {
//     subscribers: Vec<Subscriber>,
// }

// impl Dispatcher {
//     pub fn new() -> Self {
//         Self {
//             subscribers: Vec::new(),
//         }
//     }

//     pub fn add_subscriber<Event: 'static>(&mut self, action: impl Fn(&mut Event) + 'static) {
//         self.subscribers.push(Subscriber {
//             event: TypeId::of::<Event>(),
//             handler: Box::new(move |event: &mut dyn Any| {
//                 (action)(event.downcast_mut().expect("Wrong Event!"))
//             }),
//         });
//     }

//     pub fn dispatch<Event: 'static>(&self, event: &mut Event) {
//         for listener in self.subscribers.iter() {
//             if TypeId::of::<Event>() == listener.event {
//                 (listener.handler)(event);
//             }
//         }
//     }
// }
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

        dbg!("here");
        plugin.plugin.call("ping_handler", arg).unwrap();
    }
}

fn http() -> &'static Arc<HttpClient> {
    static HTTP: OnceLock<Arc<HttpClient>> = OnceLock::new();
    HTTP.get_or_init(|| Arc::new(HttpClient::new(env::var("DISCORD_TOKEN").unwrap())))
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

    // let message: Message = serde_json::from_str(&message).unwrap();
    let message = Message {
        channel_id: 1046434727978078302,
        content: "Pong!".into(),
    };

    let http = Arc::clone(&http());
    let handle = tokio::runtime::Handle::current();
    let _ = handle.enter();
    dbg!("Attempting to send message...");
    futures::executor::block_on(async {
        let res = http
            .create_message(Id::new(1046434727978078302))
            .content(&message.content)
            .expect("failed to send message")
            .await
            .expect("Failed to send message");
    });
    dbg!("Here2");

    outputs[0] = inputs[0].clone();
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the tracing subscriber.
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN")?;

    // Use intents to only receive guild message events.
    let mut shard = Shard::new(
        ShardId::ONE,
        token.clone(),
        Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT,
    );

    // Since we only care about new messages, make the cache only
    // cache new messages.
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    // let wasm = include_bytes!("../plugins/ping/target/wasm32-unknown-unknown/release/ping.wasm");

    // let f = Function::new(
    //     "send_message",
    //     [ValType::I64],
    //     [ValType::I64],
    //     None,
    //     send_message,
    // );
    // let mut plugin = extism::Plugin::create(wasm, [f], true).unwrap();
    // let output = plugin.call("init", "").unwrap();

    // let _subscribed_events: SubscribedEvents =
    //     serde_json::from_str(std::str::from_utf8(output).unwrap()).unwrap();

    // Process each event as they come in.
    loop {
        let event = match shard.next_event().await {
            Ok(event) => event,
            Err(source) => {
                tracing::warn!(?source, "error receiving event");

                if source.is_fatal() {
                    break;
                }

                continue;
            }
        };

        // Update the cache with the event.
        cache.update(&event);

        tokio::task::spawn(handle_event(event));
    }

    Ok(())
}

async fn handle_event(event: Event) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) => {
            let json = serde_json::to_string(&msg.clone()).unwrap();

            plugin_manager()
                .lock()
                .unwrap()
                .dispatch("MessageCreate", &json);
            // http.create_message(msg.channel_id)
            //     .content("Pong!")?
            //     .await?;
        }
        // Other events here...
        _ => {}
    }

    Ok(())
}
