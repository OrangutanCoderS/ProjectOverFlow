use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Logical identifier of a policy (comes from policy_tree).
pub type PolicyId = String;

/// Logical identifier of an execution plan.
/// For now, this is a simple u64; higher layers can override if needed.
pub type PlanId = u64;

/// Input from policy_tree → execution_node.
/// Pure data, fully serializable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInput {
    pub policy_id: PolicyId,
    pub context_node_id: u16,
    /// Optional process id for the context, if known.
    pub context_process_id: Option<u32>,
    /// Optional file path for the context, if known.
    pub context_file_path: Option<String>,
    /// Risk score from policy evaluation (0.0–1.0).
    pub risk_score: f32,
    /// Tags attached by policy / runtime.
    pub tags: Vec<String>,
    /// Monotonic-like timestamp when trigger occurred (ms).
    pub trigger_time_ms: u64,
}

/// High-level action class. This does NOT call any action crate by itself.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionKind {
    KillProcess,
    SuspendProcess,
    ThrottleProcess,
    FakeOutput,
    InjectEnv,
    NetworkRedirect,
    SecureModeToggle,
    /// Future extension hook.
    Custom(String),
}

/// Parameter payload for an action.
/// Tagged enum so it remains flexible without breaking config schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ActionParams {
    /// No parameters.
    Empty,
    /// Kill process, optionally with custom signal.
    Kill {
        signal: Option<i32>,
    },
    /// Suspend the process.
    Suspend,
    /// Throttle CPU usage.
    Throttle {
        cpu_limit_percent: u8,
    },
    /// Emit fake output.
    FakeOutput {
        pattern: String,
    },
    /// Inject environment variable.
    InjectEnv {
        key: String,
        value: String,
    },
    /// Redirect network to a destination.
    NetworkRedirect {
        destination: String,
    },
    /// Toggle secure mode.
    SecureModeToggle {
        enabled: bool,
    },
    /// Arbitrary extra payload.
    Custom(serde_json::Value),
}

/// How to select the runtime target from the input context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetSelector {
    /// No target (system-wide or purely logical action).
    #[serde(rename = "None")]
    None,
    /// Use the process associated with the context, if any.
    #[serde(rename = "ContextProcess")]
    ContextProcess,
    /// Use the file associated with the context, if any.
    #[serde(rename = "ContextFile")]
    ContextFile,
    /// Explicit custom target string.
    Custom { target: String },
}

/// Concrete target for a step once planned.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionTarget {
    None,
    ProcessId(u32),
    FilePath(String),
    Custom(String),
}

/// Template of an execution step, loaded from config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStepTemplate {
    pub kind: ActionKind,
    pub target: TargetSelector,
    pub params: ActionParams,
    /// If true, failure aborts subsequent steps.
    #[serde(default)]
    pub must_succeed: bool,
    /// Relative weight / importance of this step.
    #[serde(default = "default_base_weight")]
    pub base_weight: f32,
}

fn default_base_weight() -> f32 {
    1.0
}

/// Configuration: mapping from policy → step templates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfigDef {
    pub policies: HashMap<PolicyId, Vec<ExecutionStepTemplate>>,
    /// Safety cap to avoid unbounded plans.
    #[serde(default = "default_max_steps")]
    pub max_steps: usize,
}

fn default_max_steps() -> usize {
    64
}

/// A single concrete step in an execution plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub id: u16,
    pub kind: ActionKind,
    pub target: ExecutionTarget,
    pub params: ActionParams,
    pub weight: f32,
    pub must_succeed: bool,
}

/// Status of a step after execution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepStatus {
    Success,
    Skipped,
    Failed,
}

/// Result for a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStepResult {
    pub step_id: u16,
    pub status: StepStatus,
    pub error_code: Option<crate::error::ExecErrorCode>,
    pub error_message: Option<String>,
    pub duration_ms: u32,
}

/// Execution plan: deterministic, serializable description of actions to take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub id: PlanId,
    pub policy_id: PolicyId,
    pub steps: Vec<ExecutionStep>,
    pub created_at_ms: u64,
}

/// Execution report: result of running an ExecutionPlan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub plan_id: PlanId,
    pub policy_id: PolicyId,
    pub started_at_ms: u64,
    pub finished_at_ms: u64,
    pub step_results: Vec<ExecutionStepResult>,
    pub aborted: bool,
}
