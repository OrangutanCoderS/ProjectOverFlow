use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorRequest {
    pub pid: i32,
    pub sandboxed: bool,
    pub context: Option<String>,
}