use discord::Discord;
use extism::InternalExt;
use extism::{UserData, Val};
use hank_transport::Message;
use std::env;
use std::sync::{Arc, OnceLock};

pub fn discord() -> &'static Arc<Discord> {
    static DISCORD: OnceLock<Arc<Discord>> = OnceLock::new();
    DISCORD.get_or_init(|| {
        Arc::new(Discord::from_bot_token(&env::var("DISCORD_TOKEN").unwrap()).unwrap())
    })
}

pub fn send_message(
    plugin: &mut extism::CurrentPlugin,
    inputs: &[Val],
    _outputs: &mut [Val],
    _user_data: UserData,
) -> Result<(), extism::Error> {
    let message: String = plugin
        .memory_read_str(inputs[0].i64().unwrap().try_into().unwrap())
        .unwrap()
        .to_string();
    let message: Message = serde_json::from_str(&message).unwrap();

    let discord = Arc::clone(discord());
    let _ = discord.send_message(
        discord::model::ChannelId(message.channel_id),
        &message.content,
        "",
        false,
    );

    Ok(())
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
