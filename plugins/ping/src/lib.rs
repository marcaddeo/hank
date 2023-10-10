use extism_pdk::*;
use map_macro::hash_map_e;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub channel_id: u64,
    pub content: String,
}

#[host_fn]
extern "ExtismHost" {
    pub fn send_message(data: Json<Message>);
}

#[plugin_fn]
pub fn ping_handler(Json(msg): Json<Message>) -> FnResult<()> {
    info!("ping_handler");

    if msg.content == "!ping" {
        let message = Message {
            channel_id: msg.channel_id,
            content: "Pong!".into(),
        };

        unsafe {
            let _ = send_message(Json(message));
        }
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct SubscribedEvents<'a>(HashMap<&'a str, Vec<&'a str>>);

#[plugin_fn]
pub fn init(_: ()) -> FnResult<Json<SubscribedEvents<'static>>> {
    let events: HashMap<&str, Vec<&str>> = hash_map_e! {
        "MessageCreate" => vec!["ping_handler"],
    };

    Ok(Json(SubscribedEvents(events)))
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
