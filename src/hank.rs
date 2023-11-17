use crate::conf::Conf;
use crate::plugin::Plugin;
use hank_transport::HankEvent;

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

    pub async fn dispatch(&self, event: HankEvent) {
        for plugin in self.plugins.iter() {
            if plugin.subscribed_events.0.contains(&event.name) {
                plugin.handle_event(&event).await;
            }
        }
    }
}
