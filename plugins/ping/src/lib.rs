use extism_pdk::*;
use hank_transport::{HankEvent, Message, SubscribedEvents};
use serde::{Deserialize, Serialize};

#[host_fn]
extern "ExtismHost" {
    pub fn send_message(data: Json<Message>);
}

#[derive(Debug, Serialize, Deserialize)]
enum PluginResult {
    Init(SubscribedEvents),
    HandleEventResult(Option<Message>),
}

#[plugin_fn]
pub fn handle_event(Json(event): Json<HankEvent>) -> FnResult<Json<PluginResult>> {
    if event.name == "MessageCreate" {
        let payload: Message = serde_json::from_str(&event.payload).unwrap();

        if payload.content == "!ping" {
            let message = Message {
                channel_id: payload.channel_id,
                content: "Pong!".into(),
            };

            return Ok(Json(PluginResult::HandleEventResult(Some(message))));
        }
    }

    Ok(Json(PluginResult::HandleEventResult(None)))
}

#[plugin_fn]
pub fn init(_: ()) -> FnResult<Json<PluginResult>> {
    unsafe {
        let message = Message {
            channel_id: "1046434727978078302".into(),
            content: "Init!".into(),
        };
        let _ = send_message(Json(message));
    }
    Ok(Json(PluginResult::Init(SubscribedEvents(vec![
        "MessageCreate".into(),
    ]))))
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
