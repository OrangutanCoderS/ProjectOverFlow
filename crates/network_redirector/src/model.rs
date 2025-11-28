use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{Utc, DateTime};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RedirectMode {
    Block,
    Redirect,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    Domain,
    Ip,
    IpRange,
    Socket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectRequest {
    pub mode: RedirectMode,
    pub target_pid: Option<i32>,
    pub match_type: MatchType,
    pub match_value: String,
    pub redirect_to: Option<String>,
    pub ttl_secs: Option<u64>,
    pub reason: Option<String>,
    pub plugin: Option<String>,
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub dry_run: bool,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuleStatus {
    Pending,
    Active,
    Expired,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectRule {
    pub id: Uuid,
    pub request: RedirectRequest,
    pub applied_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: RuleStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectLogEntry {
    pub timestamp: DateTime<Utc>,
    pub stage: String,
    pub request: RedirectRequest,
    pub rule: Option<RedirectRule>,
    pub backend_result: Option<String>,
    pub error: Option<String>,
}