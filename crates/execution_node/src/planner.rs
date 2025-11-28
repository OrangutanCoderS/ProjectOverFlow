use crate::config::ExecutionConfig;
use crate::error::ExecutionNodeError;
use crate::model::{
    ExecutionInput, ExecutionPlan, ExecutionStep, ExecutionStepTemplate, ExecutionTarget,
    TargetSelector,
};

/// Build a concrete execution plan for the given input + configuration.
///
/// Pure function over data. No side effects.
pub fn build_plan(
    cfg: &ExecutionConfig,
    input: &ExecutionInput,
) -> Result<ExecutionPlan, ExecutionNodeError> {
    let policy_id = input.policy_id.clone();

    let templates = cfg
        .policies
        .get(&policy_id)
        .ok_or_else(|| ExecutionNodeError::MissingPolicyTemplate(policy_id.clone()))?;

    if templates.len() > cfg.max_steps {
        return Err(ExecutionNodeError::StepLimitExceeded {
            policy_id,
            attempted: templates.len(),
            max: cfg.max_steps,
        });
    }

    let mut steps = Vec::with_capacity(templates.len());

    for (i, tmpl) in templates.iter().enumerate() {
        let target = resolve_target(&tmpl.target, input);

        let step = ExecutionStep {
            id: i as u16,
            kind: tmpl.kind.clone(),
            target,
            params: tmpl.params.clone(),
            weight: tmpl.base_weight,
            must_succeed: tmpl.must_succeed,
        };

        steps.push(step);
    }

    // Plan id: currently derived from context_node_id and number of steps.
    // Higher layers can replace this when persisting.
    let plan_id: u64 =
        ((input.context_node_id as u64) << 32) ^ (steps.len() as u64) ^ (input.trigger_time_ms);

    Ok(ExecutionPlan {
        id: plan_id,
        policy_id: input.policy_id.clone(),
        steps,
        created_at_ms: input.trigger_time_ms,
    })
}

fn resolve_target(selector: &TargetSelector, input: &ExecutionInput) -> ExecutionTarget {
    match selector {
        TargetSelector::None => ExecutionTarget::None,
        TargetSelector::ContextProcess => {
            if let Some(pid) = input.context_process_id {
                ExecutionTarget::ProcessId(pid)
            } else {
                ExecutionTarget::None
            }
        }
        TargetSelector::ContextFile => {
            if let Some(path) = &input.context_file_path {
                ExecutionTarget::FilePath(path.clone())
            } else {
                ExecutionTarget::None
            }
        }
        TargetSelector::Custom { target } => ExecutionTarget::Custom(target.clone()),
    }
}
