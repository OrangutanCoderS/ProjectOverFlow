#[derive(Debug, Clone)]
pub struct ManagerConfig {
    pub default_ttl_secs: u64,
    pub max_ttl_secs: u64,
    pub require_privilege: bool,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            default_ttl_secs: 3600,
            max_ttl_secs: 86400,
            require_privilege: true,
        }
    }
}