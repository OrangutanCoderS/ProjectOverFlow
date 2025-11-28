use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WirePacket {
    pub msg: crate::model::SyncMessage,
}
