#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::mock::MockProcController;
    use crate::audit::AuditLogger;
    use crate::request::{KillRequest, KillMode};
    use std::sync::Arc;
    use tempfile::tempdir;

    #[test]
    fn test_softkill_mock() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("audit.jsonl");
        let logger = Arc::new(AuditLogger::new(log_path.to_str().unwrap()).unwrap());
        let ctrl = Arc::new(MockProcController::new());
        ctrl.insert(123);

        let mgr = ActionKillManager::new(ctrl, logger);
        let req = KillRequest::new(123, KillMode::SoftKill, "tester", false);
        let res = mgr.execute(&req).unwrap();
        assert!(res.success);
    }

    #[test]
    fn test_not_found() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("audit.jsonl");
        let logger = Arc::new(AuditLogger::new(log_path.to_str().unwrap()).unwrap());
        let ctrl = Arc::new(MockProcController::new());
        let mgr = ActionKillManager::new(ctrl, logger);
        let req = KillRequest::new(9999, KillMode::SoftKill, "tester", false);
        assert!(matches!(mgr.execute(&req), Err(_)));
    }
}