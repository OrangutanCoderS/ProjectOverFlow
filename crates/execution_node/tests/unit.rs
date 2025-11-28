use execution_node::{
    build_plan, execute_plan, ExecutionConfig, ExecutionConfigDef, ExecutionInput, NullBackend,
};

#[test]
fn plan_and_execute_single_step() {
    let cfg_json = r#"
{
  "policies": {
    "P_HIGH": [
      {
        "kind": "KillProcess",
        "target": "ContextProcess",
        "params": { "type": "Kill", "data": { "signal": 9 } },
        "must_succeed": true,
        "base_weight": 1.0
      }
    ]
  },
  "max_steps": 8
}
"#;

    let cfg: ExecutionConfigDef = serde_json::from_str(cfg_json).expect("config parses");
    let cfg: ExecutionConfig = cfg;

    let input = ExecutionInput {
        policy_id: "P_HIGH".to_string(),
        context_node_id: 1,
        context_process_id: Some(1234),
        context_file_path: None,
        risk_score: 0.9,
        tags: vec!["test".to_string()],
        trigger_time_ms: 42,
    };

    let plan = build_plan(&cfg, &input).expect("plan builds");
    assert_eq!(plan.steps.len(), 1);
    assert_eq!(plan.policy_id, "P_HIGH");

    let mut backend = NullBackend::default();
    let report = execute_plan(&plan, &input, &mut backend);

    assert_eq!(report.aborted, false);
    assert_eq!(report.step_results.len(), 1);
    assert_eq!(report.step_results[0].status, execution_node::StepStatus::Success);
}

#[test]
fn missing_policy_is_error() {
    let cfg_json = r#"
{
  "policies": {},
  "max_steps": 4
}
"#;

    let cfg: ExecutionConfigDef = serde_json::from_str(cfg_json).expect("config parses");
    let cfg: ExecutionConfig = cfg;

    let input = ExecutionInput {
        policy_id: "UNKNOWN".to_string(),
        context_node_id: 0,
        context_process_id: None,
        context_file_path: None,
        risk_score: 0.0,
        tags: vec![],
        trigger_time_ms: 0,
    };

    let err = build_plan(&cfg, &input).expect_err("should fail");
    match err {
        execution_node::ExecutionNodeError::MissingPolicyTemplate(id) => {
            assert_eq!(id, "UNKNOWN");
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
fn step_limit_enforced() {
    let cfg_json = r#"
{
  "policies": {
    "P_MANY": [
      { "kind": "KillProcess", "target": "ContextProcess", "params": { "type": "Kill", "data": { "signal": 9 } }, "must_succeed": false, "base_weight": 1.0 },
      { "kind": "KillProcess", "target": "ContextProcess", "params": { "type": "Kill", "data": { "signal": 9 } }, "must_succeed": false, "base_weight": 1.0 }
    ]
  },
  "max_steps": 1
}
"#;

    let cfg: ExecutionConfigDef = serde_json::from_str(cfg_json).expect("config parses");
    let cfg: ExecutionConfig = cfg;

    let input = ExecutionInput {
        policy_id: "P_MANY".to_string(),
        context_node_id: 0,
        context_process_id: Some(123),
        context_file_path: None,
        risk_score: 0.5,
        tags: vec![],
        trigger_time_ms: 0,
    };

    let err = build_plan(&cfg, &input).expect_err("should fail due to step limit");
    match err {
        execution_node::ExecutionNodeError::StepLimitExceeded { max, attempted, .. } => {
            assert_eq!(max, 1);
            assert_eq!(attempted, 2);
        }
        other => panic!("unexpected error: {:?}", other),
    }
}
