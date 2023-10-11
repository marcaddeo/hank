use anyhow::{bail, Result};
use clap::Parser;
use cli::{Cli, Commands, HankArgs};
use conf::Conf;
use discord::model::Event;
use discord::Discord;
use extism::InternalExt;
use extism::{UserData, Val};
use hank_transport::{HankEvent, Message};
use plugin::PluginManager;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::*;

mod cli;
mod conf;
mod plugin;

fn discord() -> &'static Arc<Discord> {
    static DISCORD: OnceLock<Arc<Discord>> = OnceLock::new();
    DISCORD.get_or_init(|| {
        Arc::new(Discord::from_bot_token(&env::var("DISCORD_TOKEN").unwrap()).unwrap())
    })
}

fn plugin_manager(config: Conf) -> &'static Mutex<PluginManager<'static>> {
    static PLUGIN_MANAGER: OnceLock<Mutex<PluginManager>> = OnceLock::new();
    PLUGIN_MANAGER.get_or_init(|| Mutex::new(PluginManager::new(config.plugins)))
}

fn send_message(
    plugin: &mut extism::CurrentPlugin,
    inputs: &[Val],
    _outputs: &mut [Val],
    _user_data: UserData,
) -> Result<(), extism::Error> {
    let message: String = plugin
        .memory_read_str(inputs[0].i64().unwrap().try_into().unwrap())
        .unwrap()
        .to_string();
    let message: Message = serde_json::from_str(&message).unwrap();

    let discord = Arc::clone(discord());
    let _ = discord.send_message(
        discord::model::ChannelId(message.channel_id),
        &message.content,
        "",
        false,
    );

    Ok(())
}

fn init(config_path: Option<PathBuf>) -> Result<()> {
    // @TODO this will overwrite an existing config with no warning.
    let config_path = conf::write_config_template(config_path)?;
    let config_path_str = match config_path.to_str() {
        Some(s) => s,
        None => bail!("Could not convert path to string"),
    };
    println!("Configuration file created: {}", config_path_str);

    Ok(())
}

fn run(args: HankArgs) -> Result<()> {
    let config = Conf::load(args.config_path)?;

    // Establish and use a websocket connection
    let (mut connection, _) = discord().connect().expect("Connect failed");
    info!("Ready.");
    loop {
        match connection.recv_event() {
            Ok(Event::MessageCreate(msg)) => {
                let event = HankEvent {
                    name: "MessageCreate".into(),
                    payload: serde_json::to_string(&msg.clone()).unwrap(),
                };

                plugin_manager(config.clone())
                    .lock()
                    .unwrap()
                    .dispatch(event);
            }
            Ok(_) => {}
            Err(discord::Error::Closed(code, body)) => {
                error!("Gateway closed on us with code {:?}: {}", code, body);
                break;
            }
            Err(err) => error!("Receive error: {:?}", err),
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    // Initialize the tracing subscriber.
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::ConfigTemplate) => conf::print_config_template(),
        Some(Commands::Init { config_path }) => init(config_path.clone())?,
        None => run(cli.args)?,
    }

    Ok(())
}
