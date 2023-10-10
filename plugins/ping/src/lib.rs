use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub channel_id: u64,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HankEvent {
    pub name: String,
    pub payload: String,
}

#[host_fn]
extern "ExtismHost" {
    pub fn send_message(data: Json<Message>);
}

#[plugin_fn]
pub fn handle_event(Json(event): Json<HankEvent>) -> FnResult<()> {
    if event.name == "MessageCreate" {
        let payload: Message = serde_json::from_str(&event.payload).unwrap();

        if payload.content == "!ping" {
            let message = Message {
                channel_id: payload.channel_id,
                content: "Pong!".into(),
            };

            unsafe {
                let _ = send_message(Json(message));
            }
        }
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct SubscribedEvents<'a>(Vec<&'a str>);

#[plugin_fn]
pub fn init(_: ()) -> FnResult<Json<SubscribedEvents<'static>>> {
    Ok(Json(SubscribedEvents(vec!["MessageCreate"])))
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
