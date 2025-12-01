use autonomous_runtime::{ActionSink, RuntimeAction, RuntimeError};
use action_kill::{ActionKillManager, KillRequest, KillMode, AuditLogger as KillLogger};
use action_suspend::{ActionSuspendManager, SuspendRequest, SuspendMode};
use action_throttle::{ActionThrottleManager, ThrottleRequest, ThrottleMode}; // Added
use std::sync::Arc;
use tracing::{info, error, warn};

pub struct MainActionDispatcher {
    kill_mgr: ActionKillManager,
    suspend_mgr: ActionSuspendManager,
    // Throttle manager is stateless (pure static methods in your crate), so we don't store it
}

impl MainActionDispatcher {
    pub fn new() -> Self {
        // [Same Kill/Suspend Init code as before...]
        // Ensure logs directory exists for audit logger
        let _ = std::fs::create_dir_all("logs");
        
        // Mock controller for kill (replace with OS specific in prod)
        let kill_controller = Arc::new(action_kill::controller::mock::MockProcController::new());
        let kill_logger = Arc::new(KillLogger::new("logs/action_kill.log").unwrap());

        Self {
            kill_mgr: ActionKillManager::new(kill_controller, kill_logger),
            suspend_mgr: ActionSuspendManager::default(),
        }
    }
}

impl ActionSink for MainActionDispatcher {
    fn submit_actions(&mut self, actions: &[RuntimeAction]) -> Result<(), RuntimeError> {
        for action in actions {
            info!(target: "dispatcher", "EXECUTE: {}/{}", action.target, action.kind);
            
            match action.target.as_str() {
                "action_kill" => {
                    let pid = action.parameters["pid"].as_i64().unwrap_or(0) as i32;
                    let req = KillRequest::new(pid, KillMode::SoftKill, "runtime", false);
                    let _ = self.kill_mgr.execute(&req);
                },
                "action_suspend" => {
                    let pid = action.parameters["pid"].as_i64().unwrap_or(0) as i32;
                    let req = SuspendRequest { pid, mode: SuspendMode::Suspend, context: None };
                    let _ = self.suspend_mgr.execute(req);
                },
                "action_throttle" => {
                    // Logic for throttling
                    let pid = action.parameters["pid"].as_i64().unwrap_or(0) as i32;
                    let intensity = action.parameters["intensity"].as_f64().unwrap_or(0.5) as f32;
                    
                    let req = ThrottleRequest {
                        pid,
                        mode: ThrottleMode::CPU,
                        intensity,
                        duration_secs: Some(5), // Throttle for 5s then auto-release
                        context: Some("high_cpu_policy".into()),
                    };
                    
                    if let Err(e) = ActionThrottleManager::execute(req) {
                        error!("Throttle failed: {}", e);
                    } else {
                        info!("Throttled PID {} at intensity {:.1}", pid, intensity);
                    }
                },
                _ => warn!("Unknown action: {}", action.target),
            }
        }
        Ok(())
    }
}