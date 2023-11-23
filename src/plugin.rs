use crate::discord;
use extism::{host_fn, UserData, ValType};
use hank_transport::{Message, PluginCommand, PluginMetadata, PluginResult};
use std::path::PathBuf;
use tokio::sync::{mpsc, oneshot};
use twilight_model::id::Id;

#[derive(Clone, Debug)]
pub struct Plugin {
    /// The plugins metadata.
    pub metadata: PluginMetadata,

    plugin_tx: mpsc::Sender<(PluginCommand, oneshot::Sender<PluginResult>)>,
}

host_fn!(send_message(message: Message) {
    let handle = tokio::runtime::Handle::current();
    handle.spawn(async move {
        discord()
            .create_message(Id::new(message.channel_id.parse().unwrap()))
            .content(&message.content)
            .unwrap()
            .await
    });
    Ok(())
});

impl Plugin {
    pub async fn new(path: PathBuf) -> Self {
        let (plugin_tx, mut plugin_rx) =
            mpsc::channel::<(PluginCommand, oneshot::Sender<PluginResult>)>(32);

        tokio::spawn(async move {
            let manifest = extism::Manifest::new(vec![path]);
            let mut plugin = extism::PluginBuilder::new(manifest)
                .with_wasi(true)
                .with_function(
                    "send_message",
                    [ValType::I64],
                    [],
                    UserData::default(),
                    send_message,
                )
                .build()
                .unwrap();

            while let Some((command, response)) = plugin_rx.recv().await {
                use PluginCommand::*;

                let result = if !plugin.function_exists(command.to_string()) {
                    PluginResult::FunctionNotFound
                } else {
                    let data: &[u8] = match command {
                        GetMetadata => plugin.call("get_metadata", "").unwrap(),
                        HandleMessage(message) => plugin
                            .call("handle_message", serde_json::to_string(&message).unwrap())
                            .unwrap(),
                    };

                    let data_str = String::from_utf8(data.to_vec()).unwrap();
                    serde_json::from_str::<PluginResult>(&data_str).unwrap()
                };

                response.send(result).unwrap();
            }
        });

        // Get the plugins metadata.
        let (resp_tx, resp_rx) = oneshot::channel();
        plugin_tx
            .send((PluginCommand::GetMetadata, resp_tx))
            .await
            .unwrap();
        let PluginResult::GetMetadata(metadata) = resp_rx.await.unwrap() else {
            panic!("Could not get plugin metadata");
        };

        Self {
            metadata,
            plugin_tx,
        }
    }

    pub async fn send_command(&self, cmd: PluginCommand) -> PluginResult {
        let (resp_tx, resp_rx) = oneshot::channel();

        self.plugin_tx.send((cmd, resp_tx)).await.unwrap();
        resp_rx.await.unwrap()
    }
}
