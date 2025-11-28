use std::sync::Arc;

use network_redirector::{
    ManagerConfig, MatchType, NetworkRedirectManager, RedirectMode, RedirectRequest, RuleStatus,
    MockBackend, ManagerError,
};

/// Utility to build a default RedirectRequest for tests.
fn make_test_request() -> RedirectRequest {
    RedirectRequest {
        mode: RedirectMode::Redirect,
        target_pid: None,
        match_type: MatchType::Domain,
        match_value: "example.com".into(),
        redirect_to: Some("127.0.0.1".into()),
        ttl_secs: Some(60),
        reason: Some("unit test".into()),
        plugin: None,
        metadata: None,
        dry_run: true, // dry-run keeps backend safe in tests
        created_by: Some("unit_test".into()),
    }
}

#[test]
fn test_submit_dry_run_rule() {
    let backend = Arc::new(MockBackend::new());
    let manager =
        NetworkRedirectManager::new(backend, ManagerConfig::default(), "/tmp/netredir_test1.log");

    let req = make_test_request();
    let rule = manager.submit(req.clone()).expect("submit should succeed");

    // Status must be pending because it's dry-run
    assert_eq!(rule.status, RuleStatus::Pending);
    assert_eq!(rule.request.match_value, "example.com");
    assert!(rule.request.dry_run);

    // The rule should still be listed in manager.index
    let rules = manager.list();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].id, rule.id);
}

#[test]
fn test_submit_real_rule_and_remove() {
    let backend = Arc::new(MockBackend::new());
    let manager =
        NetworkRedirectManager::new(backend.clone(), ManagerConfig::default(), "/tmp/netredir_test2.log");

    // Real (non-dry-run) request
    let mut req = make_test_request();
    req.dry_run = false;

    let rule = manager.submit(req.clone()).expect("real submit should succeed");
    assert_eq!(rule.status, RuleStatus::Pending);

    // Should be visible in index
    let rules = manager.list();
    assert_eq!(rules.len(), 1);

    // Remove should succeed
    manager.remove(&rule.id).expect("remove should succeed");
    let rules = manager.list();
    assert!(rules.is_empty());
}

#[test]
fn test_submit_invalid_redirect() {
    let backend = Arc::new(MockBackend::new());
    let manager =
        NetworkRedirectManager::new(backend, ManagerConfig::default(), "/tmp/netredir_test3.log");

    // Missing redirect_to for Redirect mode
    let mut req = make_test_request();
    req.redirect_to = None;

    let result = manager.submit(req);
    match result {
        Err(ManagerError::Validation(_)) => {} // expected
        _ => panic!("Expected validation error for missing redirect_to"),
    }
}