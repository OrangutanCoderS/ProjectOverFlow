use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::backend::ActionBackend;
use crate::error::ExecError;
use crate::model::{
    ExecutionInput, ExecutionPlan, ExecutionReport, ExecutionStepResult, StepStatus,
};

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_millis(0))
        .as_millis() as u64
}

/// Execute a plan using the provided backend.
/// Never panics; all backend errors are captured in the report.
pub fn execute_plan<B: ActionBackend>(
    plan: &ExecutionPlan,
    input: &ExecutionInput,
    backend: &mut B,
) -> ExecutionReport {
    let started_at_ms = now_millis();
    let mut step_results = Vec::with_capacity(plan.steps.len());
    let mut aborted = false;

    for step in &plan.steps {
        if aborted {
            step_results.push(ExecutionStepResult {
                step_id: step.id,
                status: StepStatus::Skipped,
                error_code: None,
                error_message: None,
                duration_ms: 0,
            });
            continue;
        }

        let t0 = Instant::now();
        let result: Result<(), ExecError> = backend.execute_step(step, input);
        let duration_ms = t0.elapsed().as_millis() as u32;

        match result {
            Ok(()) => {
                step_results.push(ExecutionStepResult {
                    step_id: step.id,
                    status: StepStatus::Success,
                    error_code: None,
                    error_message: None,
                    duration_ms,
                });
            }
            Err(e) => {
                if step.must_succeed {
                    aborted = true;
                }

                step_results.push(ExecutionStepResult {
                    step_id: step.id,
                    status: StepStatus::Failed,
                    error_code: Some(e.code),
                    error_message: Some(e.message),
                    duration_ms,
                });
            }
        }
    }

    let finished_at_ms = now_millis();

    ExecutionReport {
        plan_id: plan.id,
        policy_id: plan.policy_id.clone(),
        started_at_ms,
        finished_at_ms,
        step_results,
        aborted,
    }
}
