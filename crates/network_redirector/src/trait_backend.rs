use crate::{RedirectRule, BackendError};
use uuid::Uuid;

pub trait NetworkRedirectBackend: Send + Sync {
    fn apply_rule(&self, rule: &RedirectRule) -> Result<(), BackendError>;
    fn remove_rule(&self, id: &Uuid) -> Result<(), BackendError>;
    fn list_rules(&self) -> Result<Vec<RedirectRule>, BackendError>;
    fn health_check(&self) -> Result<String, BackendError>;
}