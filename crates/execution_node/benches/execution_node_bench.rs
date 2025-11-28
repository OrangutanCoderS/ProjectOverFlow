use criterion::{black_box, criterion_group, criterion_main, Criterion};
use execution_node::{
    build_plan, execute_plan, ExecutionConfigDef, ExecutionInput, NullBackend,
};

fn bench_build_and_execute(c: &mut Criterion) {
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
      },
      {
        "kind": "ThrottleProcess",
        "target": "ContextProcess",
        "params": { "type": "Throttle", "data": { "cpu_limit_percent": 50 } },
        "must_succeed": false,
        "base_weight": 0.8
      }
    ]
  },
  "max_steps": 16
}
"#;

    let cfg: ExecutionConfigDef = serde_json::from_str(cfg_json).expect("config parses");

    c.bench_function("execution_node_build_and_execute", |b| {
        b.iter(|| {
            let input = ExecutionInput {
                policy_id: "P_HIGH".to_string(),
                context_node_id: 7,
                context_process_id: Some(4321),
                context_file_path: None,
                risk_score: 0.95,
                tags: vec!["bench".to_string()],
                trigger_time_ms: 123456,
            };

            let plan = build_plan(&cfg, &input).expect("plan builds");
            let mut backend = NullBackend::default();
            let report = execute_plan(&plan, &input, &mut backend);
            black_box(report);
        });
    });
}

criterion_group!(benches, bench_build_and_execute);
criterion_main!(benches);
