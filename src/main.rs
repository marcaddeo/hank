use extism::{CurrentPlugin, Error as ExtismError, Function, Plugin, UserData, Val, ValType};
use serde::Serialize;
use std::{env, error::Error, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Event, Shard, ShardId};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;

fn send_message(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    _user_data: UserData,
) -> Result<(), ExtismError> {
    let input: String = plugin.memory_get_val(&inputs[0]).unwrap();
    println!("Hello from Rust! {} from plugin!", input);
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

    // HTTP is separate from the gateway, so create a new client.
    let http = Arc::new(HttpClient::new(token));

    // Since we only care about new messages, make the cache only
    // cache new messages.
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    let wasm = include_bytes!("../plugins/ping/target/wasm32-unknown-unknown/release/ping.wasm");

    let f = Function::new(
        "send_message",
        [ValType::I64],
        [ValType::I64],
        None,
        send_message,
    );
    let mut plugin = Plugin::new(wasm, [f], true).unwrap();
    let data: String = plugin.call("init", "").unwrap();

    dbg!(data);

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

        tokio::spawn(handle_event(event, Arc::clone(&http)));
    }

    Ok(())
}

async fn handle_event(
    event: Event,
    http: Arc<HttpClient>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) if msg.content == "!piing" => {
            http.create_message(msg.channel_id)
                .content("Pong!")?
                .await?;
        }
        // Other events here...
        _ => {}
    }

    Ok(())
}
