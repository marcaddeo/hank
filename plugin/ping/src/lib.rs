use extism_pdk::*;
use hank_transport::{HankEvent, Message, SubscribedEvents};
use serde::{Deserialize, Serialize};

#[host_fn]
extern "ExtismHost" {
    pub fn send_message(message: Json<Message>);
}

#[derive(Debug, Serialize, Deserialize)]
enum PluginResult {
    Init(SubscribedEvents),
    HandleEventResult,
}

pub fn fibonacci_reccursive(n: i32) -> u64 {
    if n < 0 {
        panic!("{} is negative!", n);
    }
    match n {
        0 => panic!("zero is not a right argument to fibonacci_reccursive()!"),
        1 | 2 => 1,
        3 => 2,
        /*
        50    => 12586269025,
        */
        _ => fibonacci_reccursive(n - 1) + fibonacci_reccursive(n - 2),
    }
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
            info!("Calculating fib to 44");
            let fib = fibonacci_reccursive(44);
            info!("done: {}", fib);
            unsafe {
                let _ = send_message(Json(message));
            }
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
