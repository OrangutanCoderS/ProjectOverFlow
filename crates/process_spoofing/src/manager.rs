use std::{collections::HashMap, sync::Arc, time::{Duration, Instant}};
use parking_lot::RwLock;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{Utc, DateTime};
use thiserror::Error;

/// Errors for spoof manager
#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("duplicate entry for pid {0}")]
    Duplicate(i32),
    #[error("not found")]
    NotFound,
    #[error("ipc failure: {0}")]
    Ipc(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpoofIdentity {
    pub real_pid: i32,
    pub fake_pid: i32,
    pub fake_ppid: i32,
    pub fake_name: Option<String>,
    pub fake_env: Option<HashMap<String, String>>,
    pub created_at: DateTime<Utc>,
    pub ttl: Option<Duration>,
}

impl SpoofIdentity {
    pub fn is_expired(&self) -> bool {
        self.ttl.map_or(false, |ttl| {
            Utc::now()
                .signed_duration_since(self.created_at)
                .num_seconds() > ttl.as_secs() as i64
        })
    }
}

#[derive(Clone)]
pub struct SpoofManager {
    index: Arc<RwLock<HashMap<i32, SpoofIdentity>>>,
}

impl SpoofManager {
    pub fn new() -> Self {
        Self { index: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub fn add_spoof(&self, s: SpoofIdentity) -> Result<(), ManagerError> {
        let mut map = self.index.write();
        if map.contains_key(&s.real_pid) {
            return Err(ManagerError::Duplicate(s.real_pid));
        }
        map.insert(s.real_pid, s);
        Ok(())
    }

    pub fn remove_spoof(&self, real_pid: i32) -> Result<(), ManagerError> {
        let mut map = self.index.write();
        map.remove(&real_pid).map(|_| ()).ok_or(ManagerError::NotFound)
    }

    pub fn query(&self, real_pid: i32) -> Option<SpoofIdentity> {
        let map = self.index.read();
        map.get(&real_pid)
            .filter(|s| !s.is_expired())
            .cloned()
    }

    pub fn cleanup_expired(&self) {
        let mut map = self.index.write();
        map.retain(|_, s| !s.is_expired());
    }

    pub fn list_all(&self) -> Vec<SpoofIdentity> {
        self.index.read().values().cloned().collect()
    }
}