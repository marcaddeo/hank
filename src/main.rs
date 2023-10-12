use anyhow::{bail, Result};
use clap::Parser;
use cli::{Cli, Commands, HankArgs};
use conf::Conf;
use discord::model::Event;
use discord::Discord;
use hank_transport::HankEvent;
use plugin::Plugin;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tracing::*;

mod cli;
mod conf;
mod functions;
mod plugin;

static DISCORD: OnceLock<Arc<Discord>> = OnceLock::new();
fn discord() -> &'static Arc<Discord> {
    DISCORD.get().expect("Discord has not been initialized")
}

pub struct Hank<'a> {
    pub config: Conf,
    pub plugins: Vec<Plugin<'a>>,
}

impl Hank<'_> {
    pub fn new(config: Conf) -> Self {
        let mut plugins: Vec<Plugin> = vec![];

        for path in config.clone().plugins {
            plugins.push(Plugin::new(path));
        }

        // Initialize the Discord global singleton.
        let discord = Discord::from_bot_token(&config.discord_token).unwrap();
        DISCORD
            .set(Arc::new(discord))
            .unwrap_or_else(|_| panic!("Unable to initialize Discord"));

        Self { config, plugins }
    }

    pub fn dispatch(&mut self, event: HankEvent) {
        for plugin in self.plugins.iter_mut() {
            if plugin.subscribed_events.0.contains(&event.name) {
                plugin.handle_event(&event);
            }
        }
    }
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

    // Initialize Hank.
    let mut hank = Hank::new(config);

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

                hank.dispatch(event);
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
