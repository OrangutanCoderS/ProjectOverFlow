use std::collections::HashMap;
use parking_lot::RwLock; // 👈 parking_lot::RwLock, no poisoning Result
use uuid::Uuid;
use crate::{NetworkRedirectBackend, RedirectRule, BackendError};

pub struct MockBackend {
    rules: RwLock<HashMap<Uuid, RedirectRule>>,
}

impl MockBackend {
    pub fn new() -> Self {
        Self { rules: RwLock::new(HashMap::new()) }
    }
}

impl NetworkRedirectBackend for MockBackend {
    fn apply_rule(&self, rule: &RedirectRule) -> Result<(), BackendError> {
        self.rules.write().insert(rule.id, rule.clone());
        Ok(())
    }

    fn remove_rule(&self, id: &Uuid) -> Result<(), BackendError> {
        let removed = self.rules.write().remove(id);
        if removed.is_none() {
            return Err(BackendError::NotFound);
        }
        Ok(())
    }

    fn list_rules(&self) -> Result<Vec<RedirectRule>, BackendError> {
        Ok(self.rules.read().values().cloned().collect())
    }

    fn health_check(&self) -> Result<String, BackendError> {
        Ok("MockBackend healthy".into())
    }
}