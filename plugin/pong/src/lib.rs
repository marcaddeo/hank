use extism_pdk::*;
use hank_transport::{Message, PluginMetadata, PluginResult, Version};

#[host_fn]
extern "ExtismHost" {
    pub fn send_message(message: Json<Message>);
}

#[plugin_fn]
pub fn get_metadata() -> FnResult<Json<PluginResult>> {
    let metadata = PluginMetadata::new(
        "pong",
        "A simple plugin that just responds with Ping! when you type !pong",
        Version::new(0, 1, 0),
        true,
    );

    Ok(Json(PluginResult::GetMetadata(metadata)))
}

#[plugin_fn]
pub fn handle_message(Json(message): Json<Message>) -> FnResult<Json<PluginResult>> {
    if message.content == "!pong" {
        let response = message.response("Ping!");
        unsafe {
            let _ = send_message(Json(response));
        }
    }

    Ok(Json(PluginResult::None))
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
