pub mod backend;
pub mod config;
pub mod error;
pub mod executor;
pub mod model;
pub mod planner;

pub use backend::{ActionBackend, NullBackend};
pub use config::ExecutionConfig;
pub use model::ExecutionConfigDef;
pub use error::{ExecError, ExecErrorCode, ExecutionNodeError};
pub use executor::execute_plan;
pub use model::{
    ActionKind, ActionParams, ExecutionInput, ExecutionPlan, ExecutionReport, ExecutionStep,
    ExecutionStepResult, ExecutionTarget, PolicyId, PlanId, StepStatus, TargetSelector,
};
pub use planner::build_plan;
