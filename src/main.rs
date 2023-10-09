use extism::InternalExt;
use extism::{
    Context, CurrentPlugin, Error as ExtismError, Function, Plugin, UserData, Val, ValType,
};
use std::sync::{Arc, OnceLock};
use std::{env, error::Error};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Event, Shard, ShardId};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;

fn http() -> &'static Arc<HttpClient> {
    static HTTP: OnceLock<Arc<HttpClient>> = OnceLock::new();
    HTTP.get_or_init(|| Arc::new(HttpClient::new(env::var("DISCORD_TOKEN").unwrap())))
}

fn send_message(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    _user_data: UserData,
) -> Result<(), ExtismError> {
    let input: String = plugin
        .memory_read_str(inputs[0].i64().unwrap().try_into().unwrap())
        .unwrap()
        .to_string();
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
    let ctx = Context::new();
    let mut plugin = Plugin::new(&ctx, wasm, [f], true).unwrap();
    let data: String = String::from_utf8(plugin.call("init", "").unwrap().to_vec()).unwrap();

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

        tokio::spawn(handle_event(event, Arc::clone(&http())));
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
