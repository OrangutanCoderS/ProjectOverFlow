//! Module 1 — Core Bootstrap (Phase I)

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::Duration,
};
use thiserror::Error;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

// Import config module (Module 2)
use config::{load_from_path, EngineConfig};

static LOGGER_INIT: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("config io: {0}")]
    ConfigIo(std::io::Error),
    #[error("config parse: {0}")]
    ConfigParse(String),
    #[error("init: {0}")]
    Init(String),
    #[error("shutdown: {0}")]
    Shutdown(String),
    #[error("signal: {0}")]
    Signal(String),
}

pub type Result<T> = std::result::Result<T, BootstrapError>;

/// Lifecycle interface implemented by each subsystem (modules 2..N)
pub trait Subsystem: Send + Sync {
    fn name(&self) -> &'static str;
    fn start(&self, cfg: &EngineConfig) -> Result<()>;
    fn stop(&self) -> Result<()>;
}

#[cfg(feature = "noop")]
#[derive(Default)]
struct NoopSubsystem(&'static str);

#[cfg(feature = "noop")]
impl Subsystem for NoopSubsystem {
    fn name(&self) -> &'static str {
        self.0
    }
    fn start(&self, _cfg: &EngineConfig) -> Result<()> {
        Ok(())
    }
    fn stop(&self) -> Result<()> {
        Ok(())
    }
}

/// Deterministic start/stop ordering for all subsystems
pub struct Supervisor {
    ordered: Vec<Arc<dyn Subsystem>>,
}

impl Supervisor {
    pub fn new() -> Self {
        Self {
            ordered: Vec::new(),
        }
    }

    pub fn push<S: Subsystem + 'static>(mut self, s: S) -> Self {
        self.ordered.push(Arc::new(s));
        self
    }

    pub fn start_all(&self, cfg: &EngineConfig) -> Result<()> {
        for s in &self.ordered {
            info!(target: "bootstrap", module = s.name(), "starting");
            if let Err(e) = s.start(cfg) {
                error!(target: "bootstrap", module = s.name(), error = %e, "failed to start");
                return Err(BootstrapError::Init(format!("{}: {e}", s.name())));
            }
        }
        Ok(())
    }

    pub fn stop_all(&self) -> Result<()> {
        for s in self.ordered.iter().rev() {
            info!(target: "bootstrap", module = s.name(), "stopping");
            if let Err(e) = s.stop() {
                error!(target: "bootstrap", module = s.name(), error = %e, "failed to stop");
                return Err(BootstrapError::Shutdown(format!("{}: {e}", s.name())));
            }
        }
        Ok(())
    }
}

pub struct Daemon {
    cfg: EngineConfig,
    sup: Supervisor,
}

impl Daemon {
    pub fn new_from_file<P: AsRef<Path>>(p: P) -> Result<Self> {
        init_tracing_default();
        let cfg: EngineConfig =
            load_from_path(p).map_err(|e| BootstrapError::ConfigParse(e.to_string()))?;
        let sup = build_supervisor();
        Ok(Self { cfg, sup })
    }

    /// Start all subsystems; return immediately (non-blocking).
    pub fn run(&self) -> Result<()> {
        info!(target: "bootstrap", "daemon run begin");
        self.sup.start_all(&self.cfg)?;
        std::thread::sleep(Duration::from_millis(10));
        Ok(())
    }

    /// Block until SIGINT/SIGTERM; then stop in reverse order.
    pub fn run_until_shutdown(&self) -> Result<()> {
        info!(target: "bootstrap", "daemon run_until_shutdown begin");
        self.sup.start_all(&self.cfg)?;

        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel::<()>();
        {
            let flag = shutdown_flag.clone();
            let tx2 = tx.clone();
            ctrlc::set_handler(move || {
                if !flag.swap(true, Ordering::SeqCst) {
                    let _ = tx2.send(());
                }
            })
            .map_err(|e| BootstrapError::Signal(e.to_string()))?;
        }

        let _ = rx
            .recv()
            .map_err(|e| BootstrapError::Signal(e.to_string()))?;
        info!(target: "bootstrap", "shutdown signal received");
        self.sup.stop_all()
    }

    pub fn shutdown(&self) -> Result<()> {
        info!(target: "bootstrap", "daemon shutdown begin");
        self.sup.stop_all()
    }

    pub fn config(&self) -> &EngineConfig {
        &self.cfg
    }
}

fn init_tracing_default() {
    let mut guard = LOGGER_INIT.lock();
    if *guard {
        return;
    }
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .compact()
        .init();
    *guard = true;
}

fn build_supervisor() -> Supervisor {
    let sup = Supervisor::new();

    #[cfg(feature = "noop")]
    let sup = sup
        .push(NoopSubsystem("log_writer"))
        .push(NoopSubsystem("telemetry_bridge"));

    #[cfg(feature = "noop")]
    let sup = sup.push(NoopSubsystem("watchdog"));

    #[cfg(feature = "noop")]
    let sup = sup
        .push(NoopSubsystem("process_monitor"))
        .push(NoopSubsystem("system_stats"))
        .push(NoopSubsystem("cpu_tracker"))
        .push(NoopSubsystem("memory_monitor"))
        .push(NoopSubsystem("gpu_tracker"))
        .push(NoopSubsystem("battery_monitor"))
        .push(NoopSubsystem("thermal_logger"));

    #[cfg(feature = "noop")]
    let sup = sup
        .push(NoopSubsystem("file_monitor"))
        .push(NoopSubsystem("network_monitor"))
        .push(NoopSubsystem("dns_logger"));

    #[cfg(feature = "noop")]
    let sup = sup
        .push(NoopSubsystem("clipboard_watcher"))
        .push(NoopSubsystem("sensor_log_reader"))
        .push(NoopSubsystem("cloud_sync_detector"))
        .push(NoopSubsystem("peripheral_monitor"));

    #[cfg(feature = "noop")]
    let sup = sup.push(NoopSubsystem("crash_detector"));

    #[cfg(feature = "noop")]
    let sup = sup
        .push(NoopSubsystem("plugin_loader"))
        .push(NoopSubsystem("plugin_api"))
        .push(NoopSubsystem("plugin_engine"));

    #[cfg(feature = "noop")]
    let sup = sup.push(NoopSubsystem("secure_mode_controller"));

    #[cfg(feature = "noop")]
    let sup = sup.push(NoopSubsystem("export_tools"));

    sup
}
