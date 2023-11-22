use crate::hank::Hank;
use anyhow::{bail, Result};
use clap::Parser;
use cli::{Cli, Commands, HankArgs};
use conf::Conf;
use hank_transport::HankEvent;
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tracing::info;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_http::Client as HttpClient;

mod cli;
mod conf;
mod hank;
mod plugin;

static DISCORD: OnceLock<Arc<HttpClient>> = OnceLock::new();
fn discord() -> &'static Arc<HttpClient> {
    DISCORD.get().expect("Discord has not been initialized")
}

static HANK: OnceLock<Hank> = OnceLock::new();
fn hank() -> &'static Hank {
    HANK.get().expect("Hank has not been initialized")
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

#[tokio::main]
async fn run(args: HankArgs) -> Result<()> {
    let config = Conf::load(args.config_path)?;

    let token = config.discord_token.clone();

    // Specify intents requesting events about things like new and updated
    // messages in a guild and direct messages.
    let intents = Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES | Intents::MESSAGE_CONTENT;

    // Create a single shard.
    let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);

    // The http client is separate from the gateway, so startup a new
    // one, also use Arc such that it can be cloned to other threads.
    let http = Arc::new(HttpClient::new(token));
    DISCORD
        .set(http.clone())
        .unwrap_or_else(|_| panic!("Unable to initialize Discord singleton."));

    // Initialize Hank.
    let hank = Hank::new(config).await;
    HANK.set(hank)
        .unwrap_or_else(|_| panic!("Unable to initialize Hank singleton."));

    // Since we only care about messages, make the cache only process messages.
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    // Startup the event loop to process each event in the event stream as they
    // come in.
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
        // Update the cache.
        cache.update(&event);

        // Spawn a new task to handle the event
        tokio::spawn(handle_event(event));
    }

    Ok(())
}

async fn handle_event(event: Event) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) => {
            let event = HankEvent {
                name: "MessageCreate".into(),
                payload: serde_json::to_string(&msg.clone()).unwrap(),
            };

            hank().dispatch(&event).await;
        }
        Event::Ready(_) => {
            info!("Shard is ready");
        }
        _ => {}
    }

    Ok(())
}

fn main() -> Result<()> {
    // Initialize the tracing subscriber.
    tracing_subscriber::fmt::fmt()
        // .with_env_filter("")
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::ConfigTemplate) => conf::print_config_template(),
        Some(Commands::Init { config_path }) => init(config_path.clone())?,
        None => run(cli.args)?,
    }

    Ok(())
}
