use action_suspend::{ActionSuspendManager, SuspendMode, SuspendRequest};

#[test]
fn test_invalid_pid_safely() {
    let manager = ActionSuspendManager::default();

    // Invalid negative PID should be rejected immediately
    let req = SuspendRequest { pid: -42, mode: SuspendMode::Suspend, context: None };
    let result = manager.execute(req);
    assert!(result.is_err(), "Negative PIDs must be rejected");

    // PID 0 should also be rejected
    let req_zero = SuspendRequest { pid: 0, mode: SuspendMode::Suspend, context: None };
    let result_zero = manager.execute(req_zero);
    assert!(result_zero.is_err(), "PID 0 should be invalid on macOS/Linux");
}