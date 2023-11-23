use extism_convert::{FromBytesOwned, ToBytes};
use serde::{Deserialize, Serialize};

pub use semver::Version;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub channel_id: String,
    pub content: String,
}

impl Message {
    pub fn response(self, content: &str) -> Self {
        Self {
            channel_id: self.channel_id,
            content: content.to_string(),
        }
    }
}

impl FromBytesOwned for Message {
    fn from_bytes_owned(data: &[u8]) -> Result<Self, anyhow::Error> {
        Ok(serde_json::from_slice(data)?)
    }
}

// @TODO this doesn't seem to be working, i can't use it in the #[host_fn] in a plugin
impl ToBytes<'_> for Message {
    type Bytes = Vec<u8>;

    fn to_bytes(&self) -> Result<Self::Bytes, anyhow::Error> {
        Ok(serde_json::to_vec(self)?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// The name of the plugin.
    name: String,

    /// A short description of the plugins functionality.
    description: String,

    /// The version of the plugin.
    version: semver::Version,

    /// Whether or not the plugin needs a database. If true, a database is created using the plugin
    /// name as the database name.
    database: bool,
}

impl PluginMetadata {
    pub fn new(name: &str, description: &str, version: semver::Version, database: bool) -> Self {
        PluginMetadata {
            name: name.to_string(),
            description: description.to_string(),
            version,
            database,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PluginCommand {
    GetMetadata,
    HandleMessage(Message),
}

impl std::fmt::Display for PluginCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PluginCommand::*;

        let command = match self {
            GetMetadata => "get_metadata",
            HandleMessage(_) => "handle_message",
        };

        write!(f, "{}", command)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PluginResult {
    None,
    FunctionNotFound,
    GetMetadata(PluginMetadata),
}

impl PluginResult {
    pub fn is_none(self) -> bool {
        matches!(self, PluginResult::None)
    }
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
