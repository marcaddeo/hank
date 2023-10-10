use extism_pdk::*;
use map_macro::hash_map_e;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub channel_id: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageOut {
    pub channel_id: u64,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MessageCreate(pub Message);

#[host_fn]
extern "ExtismHost" {
    pub fn send_message(data: Json<MessageOut>) -> String;
}

#[plugin_fn]
pub fn ping_handler(Json(msg): Json<MessageCreate>) -> FnResult<()> {
    info!("ping_handler");

    let message = MessageOut {
        channel_id: 1046434727978078302,
        content: "Pong!".into(),
    };

    unsafe {
        let _ = send_message(Json(message));
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
