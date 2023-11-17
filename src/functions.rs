use crate::discord;
use hank_transport::Message;
use extism::InternalExt;
use extism::{UserData, Val};

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
    let channel = twilight_model::id::Id::new(message.channel_id.parse().unwrap());

    let handle = tokio::runtime::Handle::current();
    handle.spawn(async move {
        discord().create_message(channel).content(&message.content).unwrap().await
    });

    Ok(())
}
