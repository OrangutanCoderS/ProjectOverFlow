use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use once_cell::sync::Lazy;
use os_shim::kill_process;

use crate::errors::ExecutionError;
use crate::model::{Action, InterventionRequest};

/// Trait implemented by each action handler.
pub trait ActionHandler: Send + Sync + 'static {
    /// Human/module name (logged).
    fn module_name(&self) -> String;
    /// Execute action. Should be short & bounded. No panics.
    fn execute(&self, req: &InterventionRequest) -> Result<(), ExecutionError>;
}

/// In-memory registry (copyable snapshot).
#[derive(Clone, Default)]
pub struct ActionRegistry {
    map: HashMap<Action, Arc<dyn ActionHandler>>,
}

impl ActionRegistry {
    pub fn with(map: HashMap<Action, Arc<dyn ActionHandler>>) -> Self {
        Self { map }
    }

    pub fn get(&self, a: &Action) -> Option<Arc<dyn ActionHandler>> {
        self.map.get(a).cloned()
    }

    pub fn register(&mut self, a: Action, h: Arc<dyn ActionHandler>) {
        self.map.insert(a, h);
    }
}

/// Global default registry — pre-seeded with safe baseline handlers.
pub static DEFAULT_REGISTRY: Lazy<RwLock<ActionRegistry>> = Lazy::new(|| {
    let mut reg = ActionRegistry::default();
    reg.register(Action::ActionKill, Arc::new(handlers::KillHandler));
    reg.register(Action::ActionSuspend, Arc::new(handlers::SuspendHandler));
    reg.register(Action::ActionThrottle, Arc::new(handlers::ThrottleHandler));
    reg.register(Action::ActionFakeOutput, Arc::new(handlers::FakeOutputHandler));
    reg.register(Action::SyscallIntercept, Arc::new(handlers::SyscallInterceptHandler));
    reg.register(Action::MemoryPatch, Arc::new(handlers::MemoryPatchHandler));
    reg.register(Action::NetworkRedirect, Arc::new(handlers::NetworkRedirectHandler));
    reg.register(Action::FakeDevice, Arc::new(handlers::FakeDeviceHandler));
    reg.register(Action::ProcessSpoof, Arc::new(handlers::ProcessSpoofHandler));
    RwLock::new(reg)
});

mod handlers {
    use super::*;
    use crate::errors::ExecutionError;

    /// NOTE: These baseline handlers are safe placeholders.
    /// They validate inputs and simulate work; project-specific
    /// heavy-lifting (ptrace/seccomp/ETW) should live in OS crates,
    /// registered here via the same trait without modifying existing code.

    pub struct KillHandler;
    impl ActionHandler for KillHandler {
        fn module_name(&self) -> String { "PluginActions/action_kill.rs".into() }
        fn execute(&self, req: &InterventionRequest) -> Result<(), ExecutionError> {
            let pid = req.target_pid
                .ok_or_else(|| ExecutionError::HandlerFailure("missing target_pid".into()))?;
            #[cfg(unix)]
            {
                if nix_kill(pid, libc::SIGKILL).is_err() {
                    return Err(ExecutionError::HandlerFailure("kill failed".into()));
                }
            }
            #[cfg(not(unix))]
            {
                return Err(ExecutionError::HandlerFailure("kill not supported on this OS".into()));
            }
            Ok(())
        }
    }

    pub struct SuspendHandler;
    impl ActionHandler for SuspendHandler {
        fn module_name(&self) -> String { "PluginActions/action_suspend.rs".into() }
        fn execute(&self, req: &InterventionRequest) -> Result<(), ExecutionError> {
            let pid = req.target_pid
                .ok_or_else(|| ExecutionError::HandlerFailure("missing target_pid".into()))?;
            #[cfg(unix)]
            {
                if nix_kill(pid, libc::SIGSTOP).is_err() {
                    return Err(ExecutionError::HandlerFailure("suspend failed".into()));
                }
            }
            #[cfg(not(unix))]
            {
                return Err(ExecutionError::HandlerFailure("suspend not supported on this OS".into()));
            }
            Ok(())
        }
    }

    pub struct ThrottleHandler;
    impl ActionHandler for ThrottleHandler {
        fn module_name(&self) -> String { "PluginActions/action_throttle.rs".into() }
        fn execute(&self, req: &InterventionRequest) -> Result<(), ExecutionError> {
            let level = req.metadata
                .get("cpu_limit_pct")
                .and_then(|v| v.as_u64())
                .unwrap_or(50);
            if !(1..=100).contains(&level) {
                return Err(ExecutionError::HandlerFailure(
                    "cpu_limit_pct out of range [1..100]".into(),
                ));
            }
            Ok(())
        }
    }

    pub struct FakeOutputHandler;
    impl ActionHandler for FakeOutputHandler {
        fn module_name(&self) -> String { "PluginActions/action_fake_output.rs".into() }
        fn execute(&self, _req: &InterventionRequest) -> Result<(), ExecutionError> {
            Ok(())
        }
    }

    pub struct SyscallInterceptHandler;
    impl ActionHandler for SyscallInterceptHandler {
        fn module_name(&self) -> String { "Augmentation/syscall_interceptor.c".into() }
        fn execute(&self, _req: &InterventionRequest) -> Result<(), ExecutionError> {
            Ok(())
        }
    }

    pub struct MemoryPatchHandler;
    impl ActionHandler for MemoryPatchHandler {
        fn module_name(&self) -> String { "Augmentation/memory_patcher.rs".into() }
        fn execute(&self, _req: &InterventionRequest) -> Result<(), ExecutionError> {
            Ok(())
        }
    }

    pub struct NetworkRedirectHandler;
    impl ActionHandler for NetworkRedirectHandler {
        fn module_name(&self) -> String { "Augmentation/network_redirector.rs".into() }
        fn execute(&self, _req: &InterventionRequest) -> Result<(), ExecutionError> {
            Ok(())
        }
    }

    pub struct FakeDeviceHandler;
    impl ActionHandler for FakeDeviceHandler {
        fn module_name(&self) -> String { "Augmentation/fake_device_emulator.rs".into() }
        fn execute(&self, _req: &InterventionRequest) -> Result<(), ExecutionError> {
            Ok(())
        }
    }

    pub struct ProcessSpoofHandler;
    impl ActionHandler for ProcessSpoofHandler {
        fn module_name(&self) -> String { "Augmentation/process_spoofing_layer.rs".into() }
        fn execute(&self, _req: &InterventionRequest) -> Result<(), ExecutionError> {
            Ok(())
        }
    }

    // Small POSIX signal helper — wraps os_shim::kill_process
    #[cfg(unix)]
    fn nix_kill(pid: i32, sig: i32) -> Result<(), ()> {
        match kill_process(pid, sig) {
            Ok(()) => Ok(()),
            Err(_) => Err(()),
        }
    }
}
