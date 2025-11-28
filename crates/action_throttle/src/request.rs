use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThrottleMode {
    CPU,
    IO,
    NET,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottleRequest {
    pub pid: i32,
    pub mode: ThrottleMode,
    pub intensity: f32,          // 0.0–1.0
    pub duration_secs: Option<u64>,
    pub context: Option<String>,
}