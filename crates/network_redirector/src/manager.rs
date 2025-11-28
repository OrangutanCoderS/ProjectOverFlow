use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use parking_lot::RwLock;
use std::collections::HashMap;

use crate::{RedirectRequest, RedirectRule, RuleStatus,
    NetworkRedirectBackend, ManagerError, ManagerConfig,
    JsonlLogger};

pub struct NetworkRedirectManager {
    backend: Arc<dyn NetworkRedirectBackend>,
    logger: JsonlLogger,
    config: ManagerConfig,
    index: RwLock<HashMap<Uuid, RedirectRule>>,
}

impl NetworkRedirectManager {
    pub fn new(backend: Arc<dyn NetworkRedirectBackend>, config: ManagerConfig, log_path: &str) -> Self {
        Self {
            backend,
            logger: JsonlLogger::new(log_path),
            config,
            index: RwLock::new(HashMap::new()),
        }
    }

    pub fn submit(&self, req: RedirectRequest) -> Result<RedirectRule, ManagerError> {
    if req.mode == crate::RedirectMode::Redirect && req.redirect_to.is_none() {
        return Err(ManagerError::Validation(crate::ValidationError::MissingRedirectTarget));
    }

    let id = Uuid::new_v4();
    let rule = RedirectRule {
        id,
        request: req.clone(),
        applied_at: Some(Utc::now()),
        expires_at: None,
        status: RuleStatus::Pending,
    };

    if req.dry_run {
        // ✅ Insert into index so tests and observability see it
        self.index.write().insert(id, rule.clone());
        self.logger.log_simple("dry-run", req, Some("skipped backend".into()), None);
        return Ok(rule);
    }

    self.backend.apply_rule(&rule)?;
    self.index.write().insert(id, rule.clone());
    self.logger.log_simple("apply", req, Some("backend success".into()), None);

    Ok(rule)
}

    pub fn remove(&self, id: &Uuid) -> Result<(), ManagerError> {
        self.backend.remove_rule(id)?;
        self.index.write().remove(id);
        Ok(())
    }

    pub fn list(&self) -> Vec<RedirectRule> {
        self.index.read().values().cloned().collect()
    }
}