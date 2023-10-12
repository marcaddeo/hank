use crate::functions::send_message;
use extism::{Function, ValType};
use hank_transport::{HankEvent, SubscribedEvents};
use std::path::PathBuf;
use tracing::error;

pub struct Plugin<'a> {
    /// A list of events the plugin subscribes to.
    pub subscribed_events: SubscribedEvents,

    pub plugin: extism::Plugin<'a>,
}

impl<'a> Plugin<'a> {
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        let f = Function::new("send_message", [ValType::I64], [], None, send_message);

        let manifest = extism::Manifest::new(vec![path.into()]);
        let mut plugin = extism::Plugin::create_with_manifest(&manifest, [f], true).unwrap();

        // Call the plugins "init" function to get a list of subscribed events.
        let output = plugin.call("init", "").unwrap();
        let subscribed_events: SubscribedEvents =
            serde_json::from_str(std::str::from_utf8(output).unwrap()).unwrap();

        Self {
            subscribed_events,
            plugin,
        }
    }

    pub fn handle_event(&mut self, event: &HankEvent) {
        let res = self
            .plugin
            .call("handle_event", serde_json::to_string(&event).unwrap());

        match res {
            Ok(_) => (),
            Err(e) => {
                error!("{}", e);
            }
        };
    }
}
