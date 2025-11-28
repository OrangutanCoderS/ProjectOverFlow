use action_throttle::{ActionThrottleManager, ThrottleMode, ThrottleRequest};

#[test]
fn test_mock_cpu_throttle() {
    let req = ThrottleRequest {
        pid: std::process::id() as i32,
        mode: ThrottleMode::CPU,
        intensity: 0.5,
        duration_secs: Some(1),
        context: Some("unit_test".into()),
    };
    let result = ActionThrottleManager::execute(req);
    assert!(result.is_ok());
}