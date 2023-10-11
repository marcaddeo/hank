use extism_pdk::*;
use hank_transport::{HankEvent, Message, SubscribedEvents};

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

#[plugin_fn]
pub fn init(_: ()) -> FnResult<Json<SubscribedEvents>> {
    Ok(Json(SubscribedEvents(vec!["MessageCreate".into()])))
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
