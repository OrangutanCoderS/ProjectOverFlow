use crate::{
    controller::ProcessController,
    errors::ManagerError,
    request::{KillRequest, KillMode, KillResult},
    audit::AuditLogger,
};
use std::sync::Arc;
use std::time::Instant;

pub struct ActionKillManager {
    controller: Arc<dyn ProcessController>,
    logger: Arc<AuditLogger>,
}

impl ActionKillManager {
    pub fn new(controller: Arc<dyn ProcessController>, logger: Arc<AuditLogger>) -> Self {
        Self { controller, logger }
    }

    pub fn execute(&self, req: &KillRequest) -> Result<KillResult, ManagerError> {
        let start = Instant::now();

        if req.dry_run {
            self.logger.log_simulated(req)?;
            return Ok(KillResult {
                request_id: req.id,
                success: true,
                message: "Dry-run only".into(),
                duration_ms: 0,
            });
        }

        if !self.controller.exists(req.target_pid) {
            return Err(ManagerError::NotFound(req.target_pid));
        }

        let signal = match req.mode {
            KillMode::SoftKill => libc::SIGTERM,
            KillMode::HardKill => libc::SIGKILL,
            KillMode::Suspend => libc::SIGSTOP,
            KillMode::Resume => libc::SIGCONT,
            KillMode::Throttle => 0,
            KillMode::Isolate => libc::SIGTERM,
            KillMode::Purge => libc::SIGKILL,
        };

        if signal != 0 {
            self.controller.send_signal(req.target_pid, signal)?;
        }

        let dur = start.elapsed().as_millis();
        self.logger.log_result(req, true, dur)?;
        Ok(KillResult {
            request_id: req.id,
            success: true,
            message: format!("Signal {} sent", signal),
            duration_ms: dur,
        })
    }
}