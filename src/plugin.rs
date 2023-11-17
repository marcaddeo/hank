// use crate::functions::send_message;
// use extism::{Function, ValType};
use hank_transport::{HankEvent, SubscribedEvents, Message};
use std::path::PathBuf;
use tokio::sync::{oneshot, mpsc};
use serde::{Serialize, Deserialize};
// use tracing::error;

#[derive(Debug, Serialize, Deserialize)]
enum PluginCommand {
    Init,
    HandleEvent(HankEvent),
}

#[derive(Debug, Serialize, Deserialize)]
enum PluginResult {
    Init(SubscribedEvents),
    HandleEventResult(Option<Message>),
}

#[derive(Clone, Debug)]
pub struct Plugin {
    /// A list of events the plugin subscribes to.
    pub subscribed_events: SubscribedEvents,
    plugin_tx: mpsc::Sender<(PluginCommand, oneshot::Sender<PluginResult>)>,
}

impl Plugin {
    pub async fn new(path: PathBuf) -> Self {
        let (plugin_tx, mut plugin_rx) = mpsc::channel::<(PluginCommand, oneshot::Sender<PluginResult>)>(32);

        tokio::spawn(async move {
            use PluginCommand::*;

            let manifest = extism::Manifest::new(vec![path]);
            let mut plugin = extism::Plugin::create_with_manifest(&manifest, [], true).unwrap();

            while let Some((command, response)) = plugin_rx.recv().await {
                let data = match command {
                    Init => {
                        plugin.call("init", "").unwrap()
                    }
                    HandleEvent(event) => {
                        plugin.call("handle_event", serde_json::to_string(&event).unwrap()).unwrap()
                    }
                };

                let data_str = String::from_utf8(data.to_vec()).unwrap();
                let result: PluginResult = serde_json::from_str(&data_str).unwrap();

                response.send(result).unwrap();
            }
        });

        // Call the plugins "init" function to get a list of subscribed events.
        let PluginResult::Init(subscribed_events) = Self::call(plugin_tx.clone(), PluginCommand::Init).await else {
            panic!("Init failed");
        };

        Self {
            subscribed_events,
            plugin_tx,
        }
    }

    pub async fn handle_event(&self, event: &HankEvent) -> Option<Message> {
        match Self::call(self.plugin_tx.clone(), PluginCommand::HandleEvent(event.clone())).await {
            PluginResult::HandleEventResult(res) => return res,
            _ => panic!("error"),
        }
    }

    async fn call(plugin_tx: mpsc::Sender<(PluginCommand, oneshot::Sender<PluginResult>)>, cmd: PluginCommand) -> PluginResult {
        let (resp_tx, resp_rx) = oneshot::channel();

        plugin_tx.send((cmd, resp_tx)).await.unwrap();
        resp_rx.await.unwrap()
    }
}
