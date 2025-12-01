use crate::{Subsystem, DaemonContext, Result, BootstrapError};
use config::EngineConfig;
use autonomous_runtime::ModuleEvent;
use std::thread;
use std::time::Duration;

// --- Imports for Monitors ---
use system::{SystemStatsMonitor, SysStatsCfg};
use filemon::{FileMonitor, FileMonCfg};
use netmon::{NetworkConfig, spawn_polling as spawn_netmon};

pub struct MonitorSupervisor;

impl Subsystem for MonitorSupervisor {
    fn name(&self) -> &'static str { "monitor_supervisor" }

    fn start(&self, _cfg: &EngineConfig, ctx: &DaemonContext) -> Result<()> {
        let tx = ctx.event_tx.clone();

        // 1. System Stats Monitor (Polling)
        let tx_sys = tx.clone();
        thread::Builder::new().name("mon-system".into()).spawn(move || {
            let mut monitor = SystemStatsMonitor::new(SysStatsCfg {
                refresh_processes: true,
                soft_budget_ms: 100,
            }).expect("Failed to init system stats");

            loop {
                if let Ok(core_event) = monitor.snapshot() {
                    let _ = tx_sys.send(ModuleEvent {
                        source: "system_stats".into(),
                        kind: "snapshot".into(),
                        timestamp: chrono::Utc::now(),
                        payload: serde_json::to_value(core_event).unwrap_or_default(),
                    });
                }
                thread::sleep(Duration::from_millis(1000));
            }
        }).map_err(|e| BootstrapError::Init(e.to_string()))?;

        // 2. File Monitor (Polling Snapshot)
        #[cfg(target_os = "macos")] // Only run on macOS for Phase I
        {
            let tx_file = tx.clone();
            thread::Builder::new().name("mon-file".into()).spawn(move || {
                let monitor = FileMonitor::new(FileMonCfg::default())
                    .expect("Failed to init file monitor");

                loop {
                    // FileMon snapshot returns a Vec<AnyEvent>
                    if let Ok(events) = monitor.snapshot() {
                        for event in events {
                            let _ = tx_file.send(ModuleEvent {
                                source: "filemon".into(),
                                kind: "access".into(),
                                timestamp: chrono::Utc::now(),
                                payload: serde_json::to_value(event).unwrap_or_default(),
                            });
                        }
                    }
                    thread::sleep(Duration::from_millis(500));
                }
            }).map_err(|e| BootstrapError::Init(e.to_string()))?;
        }

        // 3. Network Monitor (Streaming)
        let tx_net = tx.clone();
        thread::Builder::new().name("mon-network".into()).spawn(move || {
            let rx_net = spawn_netmon(NetworkConfig::default());
            
            // Loop forever reading from the netmon internal thread
            while let Ok(conn_info) = rx_net.recv() {
                let _ = tx_net.send(ModuleEvent {
                    source: "netmon".into(),
                    kind: "connection".into(),
                    timestamp: chrono::Utc::now(),
                    payload: serde_json::to_value(conn_info).unwrap_or_default(),
                });
            }
        }).map_err(|e| BootstrapError::Init(e.to_string()))?;

        Ok(())
    }

    fn stop(&self) -> Result<()> { Ok(()) }
}