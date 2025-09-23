use serde::{Deserialize, Serialize};
use crate::errors::ValidationError;
use crate::util::{utc_timestamp, ProtectedPidPolicy};

/// Supported actions (v1). Extend without breaking wire format by adding variants at end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    ActionKill,
    ActionSuspend,
    ActionThrottle,
    ActionFakeOutput,
    SyscallIntercept,
    MemoryPatch,
    NetworkRedirect,
    FakeDevice,
    ProcessSpoof,
}

/// Allow easy conversion from &str into Action (for string-based APIs).
impl TryFrom<&str> for Action {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "action_kill" => Ok(Action::ActionKill),
            "action_suspend" => Ok(Action::ActionSuspend),
            "action_throttle" => Ok(Action::ActionThrottle),
            "action_fake_output" => Ok(Action::ActionFakeOutput),
            "syscall_intercept" => Ok(Action::SyscallIntercept),
            "memory_patch" => Ok(Action::MemoryPatch),
            "network_redirect" => Ok(Action::NetworkRedirect),
            "fake_device" => Ok(Action::FakeDevice),
            "process_spoof" => Ok(Action::ProcessSpoof),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionRequest {
    /// Which action to perform (maps to a handler).
    pub action: Action,
    /// Optional target PID (some actions may not require).
    pub target_pid: Option<i32>,
    /// Optional metadata bag (path, domain, throttle%, etc.).
    #[serde(default)]
    pub metadata: serde_json::Map<String, serde_json::Value>,
    /// Optional source/plugin tag (for audit).
    #[serde(default)]
    pub plugin: Option<String>,
}

impl InterventionRequest {
    /// Validate semantics and protected PID constraints.
    pub fn validate(&self, protect: &ProtectedPidPolicy) -> Result<(), ValidationError> {
        if let Some(pid) = self.target_pid {
            if pid <= 0 {
                return Err(ValidationError::BadPid(pid));
            }
            if protect.is_protected(pid) {
                return Err(ValidationError::ProtectedPid(pid));
            }
        }
        Ok(())
    }

    /// Convenience constructor.
    pub fn new<A: Into<Action>>(
        action: A,
        target_pid: Option<i32>,
        metadata: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Self {
        Self {
            action: action.into(),
            target_pid,
            metadata: metadata.unwrap_or_default(),
            plugin: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionResult {
    pub target_pid: i32,
    pub action: Action,
    pub module: String,
    pub result: String,     // "success" | "failure" | "dry-run"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub timestamp: String,  // UTC ISO-8601
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
}

impl InterventionResult {
    pub fn success(req: &InterventionRequest, module: String) -> Self {
        Self::base(req, module, "success".into(), None)
    }
    pub fn dry_run(req: &InterventionRequest, module: String) -> Self {
        Self::base(req, module, "dry-run".into(), None)
    }
    pub fn failure(req: &InterventionRequest, stage: &str, msg: &str) -> Self {
        Self::base(req, format!("stage:{stage}"), "failure".into(), Some(msg.into()))
    }
    pub fn exec_failure(req: &InterventionRequest, module: String, msg: &str) -> Self {
        Self::base(req, module, "failure".into(), Some(msg.into()))
    }

    fn base(
        req: &InterventionRequest,
        module: String,
        result: String,
        error: Option<String>,
    ) -> Self {
        Self {
            target_pid: req.target_pid.unwrap_or_default(),
            action: req.action,
            module,
            result,
            error,
            timestamp: utc_timestamp(),
            plugin: req.plugin.clone(),
            metadata: if req.metadata.is_empty() {
                None
            } else {
                Some(req.metadata.clone())
            },
        }
    }
}
