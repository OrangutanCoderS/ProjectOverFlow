//! Unit tests for behavior_injector

use behavior_injector::{
    BehaviorInjector, InjectorConfig, InterventionRequest, Action,
    install_global_injector, dispatch_via_global, ProtectedPidPolicy,
};
use tempfile::tempdir;
use serde_json::json;

#[test]
fn validate_and_dry_run() {
    let tmp = tempdir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();

    let cfg = InjectorConfig {
        dry_run: true,
        unify_logs: false,
        protected_policy: ProtectedPidPolicy::default(),
    };
    let inj = BehaviorInjector::new_default(cfg).unwrap();

    let mut metadata = serde_json::Map::new();
    metadata.insert("reason".into(), json!("test"));

    let req = InterventionRequest::new(Action::ActionSuspend, Some(1234), Some(metadata));

    let res = inj.dispatch_intervention(req.clone());
    assert_eq!(res.result, "dry-run");
    assert_eq!(res.action, Action::ActionSuspend);
    assert_eq!(res.target_pid, 1234);
    assert!(std::path::Path::new("logs/intervention_log.json").exists());
}

#[test]
fn protected_pid_rejected() {
    let tmp = tempdir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();

    let cfg = InjectorConfig {
        dry_run: false,
        unify_logs: false,
        protected_policy: ProtectedPidPolicy { protected: vec![0, 1, 2222] },
    };
    let inj = BehaviorInjector::new_default(cfg).unwrap();

    let req = InterventionRequest::new(Action::ActionKill, Some(2222), None);
    let res = inj.dispatch_intervention(req);

    assert_eq!(res.result, "failure");
    assert!(res.error.unwrap_or_default().contains("protected"));
}

#[test]
fn global_injector_roundtrip() {
    let tmp = tempdir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();

    let inj = BehaviorInjector::new_default(InjectorConfig::default()).unwrap();
    install_global_injector(inj);

    let mut metadata = serde_json::Map::new();
    metadata.insert("cpu_limit_pct".into(), json!(30));

    let req = InterventionRequest::new(Action::ActionThrottle, Some(9001), Some(metadata));
    let res = dispatch_via_global(req).unwrap();

    assert_eq!(res.action, Action::ActionThrottle);
}
