use std::collections::VecDeque;
use std::time::Duration;
use chrono::Utc;

use crate::clock::RuntimeClock;
use crate::config::RuntimeConfig;
use crate::error::RuntimeError;
use crate::event::{ModuleEvent, RuntimeAction};
use crate::sink::ActionSink;
use crate::source::EventSource;
use crate::state::{RuntimeState, RuntimeStatus};

#[derive(Debug, Clone)]
pub enum RuntimeCommand {
    Start,
    Stop,
    Pause,
    Resume,
    Step,
    Reconfigure(RuntimeConfig),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct RuntimeTickStats {
    pub tick_number: u64,
    pub events_polled: usize,
    pub events_processed: usize,
    pub actions_emitted: usize,
    pub flushed: bool,
}

pub struct AutonomousRuntime<S, A, C>
where
    S: EventSource,
    A: ActionSink,
    C: RuntimeClock,
{
    config: RuntimeConfig,
    state: RuntimeState,
    source: S,
    sink: A,
    clock: C,
    queue: VecDeque<ModuleEvent>,
}

impl<S, A, C> AutonomousRuntime<S, A, C>
where
    S: EventSource,
    A: ActionSink,
    C: RuntimeClock,
{
    pub fn new(config: RuntimeConfig, source: S, sink: A, clock: C) -> Result<Self, RuntimeError> {
        config.validate().map_err(RuntimeError::InvalidConfig)?;
        Ok(Self {
            config,
            state: RuntimeState::new(),
            source,
            sink,
            clock,
            queue: VecDeque::new(),
        })
    }

    pub fn state(&self) -> &RuntimeState { &self.state }
    pub fn config(&self) -> &RuntimeConfig { &self.config }

    pub fn inject_event(&mut self, mut event: ModuleEvent) {
        if event.timestamp.timestamp_millis() == 0 {
            event.timestamp = Utc::now();
        }
        self.queue.push_back(event);
    }

    pub fn apply_command(&mut self, cmd: RuntimeCommand) -> Result<(), RuntimeError> {
        match cmd {
            RuntimeCommand::Start => self.state.status = RuntimeStatus::Running,
            RuntimeCommand::Stop => self.state.status = RuntimeStatus::Stopped,
            RuntimeCommand::Pause => self.state.status = RuntimeStatus::Paused,
            RuntimeCommand::Resume => self.state.status = RuntimeStatus::Running,
            RuntimeCommand::Step => { self.tick_once().map(|_| ())?; }
            RuntimeCommand::Reconfigure(cfg) => {
                cfg.validate().map_err(RuntimeError::InvalidConfig)?;
                self.config = cfg;
            }
        }
        Ok(())
    }

    pub fn tick_once(&mut self) -> Result<RuntimeTickStats, RuntimeError> {
        match self.state.status {
            RuntimeStatus::Running => {}
            RuntimeStatus::Paused => return Err(RuntimeError::Paused),
            status => return Err(RuntimeError::NotRunning(status)),
        }

        let tick_number = self.state.tick_counter + 1;

        // 1) Poll Source
        let polled_events = self.source.poll_events(self.config.max_events_per_tick)
            .map_err(|e| self.handle_error(e))?;
        
        for ev in polled_events {
            self.queue.push_back(ev);
        }

        // 2) Process Events & Apply Logic
        let mut processed = 0usize;
        let mut emitted_actions: Vec<RuntimeAction> = Vec::new();

        while processed < self.config.max_events_per_tick {
            let event = match self.queue.pop_front() {
                Some(ev) => ev,
                None => break,
            };

            // --- REACTIVE POLICY LOGIC ---
            // A simple hardcoded policy: "Throttle any process > 80% CPU"
            
            // Note: In a real system, you'd deserialize payload to core::SystemStatSnapshot
            // or core::ProcessInfo. Here we use raw JSON access for flexibility.
            
            // Check process list from System Stats
            /*
               Assumption: The system_stats module sends a "snapshot" event.
               Since Phase I system stats is mostly global, we'll pretend we receive 
               a separate per-process event or global CPU alert.
            */

            // Simple Logic: If "cpu_pct" > 80.0 in payload, trigger global alert log
            if let Some(cpu) = event.payload.get("cpu_pct").and_then(|v| v.as_f64()) {
                if cpu > 0.0 {
                    tracing::warn!("Global CPU Alert: {:.1}%", cpu);
                }
            }

            // Example Logic: React to a specific Plugin Trigger (e.g. from Memory Monitor)
            if event.source == "memory_monitor" && event.kind == "trigger" {
                tracing::warn!("Memory Trigger Received: {:?}", event.payload);
                // We could emit a "kill" action here if it was critical
            }

            processed += 1;
        }

        // 3) Submit Actions
        if !emitted_actions.is_empty() {
            self.sink.submit_actions(&emitted_actions)
                .map_err(|e| self.handle_error(e))?;
        }

        self.state.tick_counter = tick_number;
        let stats = RuntimeTickStats {
            tick_number,
            events_polled: 0, // Simplified for this view
            events_processed: processed,
            actions_emitted: emitted_actions.len(),
            flushed: false,
        };

        // Sleep
        let sleep_duration = Duration::from_millis(self.config.tick_interval_ms);
        self.clock.sleep(sleep_duration);

        Ok(stats)
    }

    fn handle_error(&mut self, err: RuntimeError) -> RuntimeError {
        if self.config.fail_fast {
            self.state.record_error(&err);
        }
        err
    }
}