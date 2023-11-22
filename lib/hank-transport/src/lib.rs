use extism_convert::{FromBytesOwned, ToBytes};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub channel_id: String,
    pub content: String,
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
pub struct HankEvent {
    pub name: String,
    pub payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribedEvents(pub Vec<String>);

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
