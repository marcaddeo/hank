use crate::conf::Conf;
use crate::plugin::Plugin;
use hank_transport::{Message, PluginCommand};
use tokio::task::JoinSet;

#[derive(Clone)]
pub struct Hank {
    pub config: Conf,
    pub plugins: Vec<Plugin>,
}

impl Hank {
    pub async fn new(config: Conf) -> Self {
        let mut plugins: Vec<Plugin> = vec![];

        for path in config.clone().plugins {
            plugins.push(Plugin::new(path).await);
        }

        Self { config, plugins }
    }

    pub async fn handle_message(&'static self, message: &Message) {
        self.send_command(PluginCommand::HandleMessage(message.clone()))
            .await;
    }

    async fn send_command(&'static self, command: PluginCommand) {
        let mut set = JoinSet::new();

        for plugin in self.plugins.iter() {
            set.spawn(plugin.send_command(command.clone()));
        }

        while (set.join_next().await).is_some() {}
    }
}
