use crate::{Subsystem, DaemonContext, Result, BootstrapError};
use config::EngineConfig;
use autonomous_runtime::{
    AutonomousRuntime, RuntimeConfig, SystemClock, RuntimeCommand, 
    ModuleEvent, RuntimeError
};
use autonomous_runtime::source::EventSource;
use crate::dispatcher::MainActionDispatcher;
use std::thread;
use std::sync::mpsc::Receiver;

pub struct RuntimeSubsystem;

impl Subsystem for RuntimeSubsystem {
    fn name(&self) -> &'static str { "autonomous_runtime" }

    fn start(&self, cfg: &EngineConfig, ctx: &DaemonContext) -> Result<()> {
        // Claim the receiver channel
        let rx = ctx.event_rx.lock().take()
            .ok_or_else(|| BootstrapError::Init("Event receiver already claimed".into()))?;
        
        let source = ChannelEventSource { rx };
        let sink = MainActionDispatcher::new();
        let clock = SystemClock;
        
        // Configure runtime from global config
        let runtime_cfg = RuntimeConfig {
            tick_interval_ms: cfg.scheduler.tick_ms,
            max_events_per_tick: 100,
            flush_interval_ticks: 50,
            fail_fast: false, // Keep running even if errors occur
        };

        let mut runtime = AutonomousRuntime::new(runtime_cfg, source, sink, clock)
            .map_err(|e| BootstrapError::Init(e.to_string()))?;

        // Start the runtime state
        runtime.apply_command(RuntimeCommand::Start)
            .map_err(|e| BootstrapError::Init(e.to_string()))?;

        // Run the tick loop in a background thread
        thread::Builder::new()
            .name("runtime-loop".into())
            .spawn(move || {
                tracing::info!(target: "runtime", "Runtime loop started");
                loop {
                    if let Err(e) = runtime.tick_once() {
                        tracing::error!(target: "runtime", "Tick error: {}", e);
                    }
                }
            })
            .map_err(|e| BootstrapError::Init(e.to_string()))?;

        Ok(())
    }

    fn stop(&self) -> Result<()> {
        // In a full implementation, you would send a Stop command via channel here
        Ok(())
    }
}

/// Adapter to make an mpsc Receiver look like an EventSource
struct ChannelEventSource {
    rx: Receiver<ModuleEvent>,
}

impl EventSource for ChannelEventSource {
    fn poll_events(&mut self, max: usize) -> std::result::Result<Vec<ModuleEvent>, RuntimeError> {
        let mut events = Vec::with_capacity(max);
        // Non-blocking fetch up to `max` events
        for _ in 0..max {
            match self.rx.try_recv() {
                Ok(ev) => events.push(ev),
                Err(_) => break, // Empty or disconnected
            }
        }
        Ok(events)
    }
}