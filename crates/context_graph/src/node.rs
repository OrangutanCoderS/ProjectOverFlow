use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKey {
    Process(String),
    FilePath(String),
    Custom(String),
    Name(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMeta {
    pub priority: u8,
    pub entropy: f32,
    pub sensitivity: u8,
    pub reactiveness: u8,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: u16,
    pub key: NodeKey,
    pub meta: NodeMeta,
}