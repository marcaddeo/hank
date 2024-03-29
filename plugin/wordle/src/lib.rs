use extism_pdk::*;
use hank_transport::{Message, PluginMetadata, PluginResult, Version};
use wordle::Puzzle;

mod wordle;

#[host_fn]
extern "ExtismHost" {
    pub fn send_message(message: Json<Message>);
    pub fn db_query(query: String) -> String;
}

#[plugin_fn]
pub fn get_metadata() -> FnResult<Json<PluginResult>> {
    let metadata = PluginMetadata::new(
        "wordle",
        "A wordle plugin to record daily Wordle puzzles.",
        Version::new(0, 1, 0),
        true,
    );

    Ok(Json(PluginResult::GetMetadata(metadata)))
}

#[plugin_fn]
pub fn handle_message(Json(message): Json<Message>) -> FnResult<Json<PluginResult>> {
    let Ok(puzzle) = Puzzle::try_from(message.content.clone()) else {
        return Ok(Json(PluginResult::None));
    };

    unsafe {
        let response = message.clone().response(&format!("{:?}", puzzle));
        let _ = send_message(Json(response));

        let res = db_query("SELECT 'hello world'".into()).unwrap();
        let response = message.response(&format!("db result: {:?}", res));
        let _ = send_message(Json(response));
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
