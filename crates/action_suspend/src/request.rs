use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuspendMode {
    Suspend,
    Resume,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspendRequest {
    pub pid: i32,
    pub mode: SuspendMode,
    pub context: Option<String>,
}