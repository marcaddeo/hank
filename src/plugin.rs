use crate::discord;
use extism::{host_fn, UserData, ValType};
use hank_transport::Message;
use hank_transport::{HankEvent, SubscribedEvents};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::{mpsc, oneshot};
use twilight_model::id::Id;

#[derive(Debug, Serialize, Deserialize)]
enum PluginCommand {
    Init,
    HandleEvent(HankEvent),
}

#[derive(Debug, Serialize, Deserialize)]
enum PluginResult {
    Init(SubscribedEvents),
    HandleEventResult,
}

#[derive(Clone, Debug)]
pub struct Plugin {
    /// A list of events the plugin subscribes to.
    pub subscribed_events: SubscribedEvents,
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
            use PluginCommand::*;

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
                let data: &[u8] = match command {
                    Init => plugin.call("init", "").unwrap(),
                    HandleEvent(event) => plugin
                        .call("handle_event", serde_json::to_string(&event).unwrap())
                        .unwrap(),
                };

                let data_str = String::from_utf8(data.to_vec()).unwrap();
                let result: PluginResult = serde_json::from_str(&data_str).unwrap();

                response.send(result).unwrap();
            }
        });

        // Call the plugins "init" function to get a list of subscribed events.
        let PluginResult::Init(subscribed_events) =
            Self::call(plugin_tx.clone(), PluginCommand::Init).await
        else {
            panic!("Init failed");
        };

        Self {
            subscribed_events,
            plugin_tx,
        }
    }

    pub async fn handle_event(&self, event: HankEvent) {
        Self::call(self.plugin_tx.clone(), PluginCommand::HandleEvent(event)).await;
    }

    async fn call(
        plugin_tx: mpsc::Sender<(PluginCommand, oneshot::Sender<PluginResult>)>,
        cmd: PluginCommand,
    ) -> PluginResult {
        let (resp_tx, resp_rx) = oneshot::channel();

        plugin_tx.send((cmd, resp_tx)).await.unwrap();
        resp_rx.await.unwrap()
    }
}
