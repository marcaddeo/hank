use extism_pdk::*;
use hank_transport::{HankEvent, Message, SubscribedEvents};
use serde::{Deserialize, Serialize};

mod wordle;

#[host_fn]
extern "ExtismHost" {
    pub fn send_message(message: Json<Message>);
}

#[derive(Debug, Serialize, Deserialize)]
enum PluginResult {
    Init(SubscribedEvents),
    HandleEventResult,
}

#[plugin_fn]
pub fn handle_event(Json(event): Json<HankEvent>) -> FnResult<Json<PluginResult>> {
    if event.name == "MessageCreate" {
        let payload: Message = serde_json::from_str(&event.payload).unwrap();

        let Ok(puzzle) = wordle::Puzzle::try_from(payload.content.clone()) else {
            return Ok(Json(PluginResult::HandleEventResult));
        };

        let message = Message {
            channel_id: payload.channel_id,
            content: format!("{:?}", puzzle),
        };
        unsafe {
            let _ = send_message(Json(message));
        }
    }

    Ok(Json(PluginResult::HandleEventResult))
}

#[plugin_fn]
pub fn init() -> FnResult<Json<PluginResult>> {
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
