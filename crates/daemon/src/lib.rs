//! Module 1 — Core Bootstrap (Phase I) - Integrated

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Sender, Receiver},
        Arc,
    },
    time::Duration,
};
use thiserror::Error;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

use config::{load_from_path, EngineConfig};
use autonomous_runtime::ModuleEvent;

// Module declarations
pub mod dispatcher;
pub mod runtime_subsystem;
pub mod monitor_subsystem;

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

/// Shared context for all subsystems
pub struct DaemonContext {
    pub event_tx: Sender<ModuleEvent>,
    pub event_rx: Mutex<Option<Receiver<ModuleEvent>>>,
}

impl DaemonContext {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            event_tx: tx,
            event_rx: Mutex::new(Some(rx)),
        }
    }
}

pub trait Subsystem: Send + Sync {
    fn name(&self) -> &'static str;
    fn start(&self, cfg: &EngineConfig, ctx: &DaemonContext) -> Result<()>;
    fn stop(&self) -> Result<()>;
}

pub struct Supervisor {
    ordered: Vec<Arc<dyn Subsystem>>,
    ctx: Arc<DaemonContext>,
}

impl Supervisor {
    pub fn new(ctx: Arc<DaemonContext>) -> Self {
        Self {
            ordered: Vec::new(),
            ctx,
        }
    }

    pub fn push<S: Subsystem + 'static>(mut self, s: S) -> Self {
        self.ordered.push(Arc::new(s));
        self
    }

    pub fn start_all(&self, cfg: &EngineConfig) -> Result<()> {
        for s in &self.ordered {
            info!(target: "bootstrap", module = s.name(), "starting");
            if let Err(e) = s.start(cfg, &self.ctx) {
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
    ctx: Arc<DaemonContext>,
}

impl Daemon {
    pub fn new_from_file<P: AsRef<Path>>(p: P) -> Result<Self> {
        init_tracing_default();
        let cfg = load_from_path(p).map_err(|e| BootstrapError::ConfigParse(e.to_string()))?;
        let ctx = Arc::new(DaemonContext::new());
        let sup = build_supervisor(ctx.clone());
        Ok(Self { cfg, sup, ctx })
    }

    pub fn run(&self) -> Result<()> {
        info!(target: "bootstrap", "daemon run begin");
        self.sup.start_all(&self.cfg)?;
        std::thread::sleep(Duration::from_millis(100));
        Ok(())
    }

    pub fn run_until_shutdown(&self) -> Result<()> {
        info!(target: "bootstrap", "daemon run_until_shutdown begin");
        self.sup.start_all(&self.cfg)?;

        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        {
            let flag = shutdown_flag.clone();
            let tx2 = tx.clone();
            if let Err(e) = ctrlc::set_handler(move || {
                if !flag.swap(true, Ordering::SeqCst) { let _ = tx2.send(()); }
            }) {
                error!("Failed to set Ctrl+C handler: {}", e);
            }
        }
        let _ = rx.recv();
        info!(target: "bootstrap", "shutdown signal received");
        self.sup.stop_all()
    }
}

fn init_tracing_default() {
    let mut guard = LOGGER_INIT.lock();
    if *guard { return; }
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).with_target(true).compact().init();
    *guard = true;
}

fn build_supervisor(ctx: Arc<DaemonContext>) -> Supervisor {
    let sup = Supervisor::new(ctx);
    // 1. Brain First
    let sup = sup.push(crate::runtime_subsystem::RuntimeSubsystem);
    // 2. Eyes Second (Combined Monitor Subsystem)
    let sup = sup.push(crate::monitor_subsystem::MonitorSupervisor);
    sup
}
