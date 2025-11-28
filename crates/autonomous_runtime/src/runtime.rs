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

/// Commands that external code can send to the runtime.
///
/// In Phase III integration, these will come from IPC / CLI / GUI.
#[derive(Debug, Clone)]
pub enum RuntimeCommand {
    Start,
    Stop,
    Pause,
    Resume,
    /// Advance exactly one tick (used in tests / deterministic modes).
    Step,
    /// Replace configuration at runtime.
    Reconfigure(RuntimeConfig),
}

/// Statistics for a single tick, useful for tests and debugging.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct RuntimeTickStats {
    pub tick_number: u64,
    pub events_polled: usize,
    pub events_processed: usize,
    pub actions_emitted: usize,
    pub flushed: bool,
}

/// Single-threaded autonomous runtime.
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
    /// Pending events that were injected or polled but not yet processed.
    queue: VecDeque<ModuleEvent>,
}

impl<S, A, C> AutonomousRuntime<S, A, C>
where
    S: EventSource,
    A: ActionSink,
    C: RuntimeClock,
{
    /// Create a new runtime with the given components.
    ///
    /// This does *not* start the loop; you must call `apply_command(Start)`
    /// or manually manipulate the state in tests.
    pub fn new(config: RuntimeConfig, source: S, sink: A, clock: C) -> Result<Self, RuntimeError> {
        config
            .validate()
            .map_err(RuntimeError::InvalidConfig)?;

        Ok(Self {
            config,
            state: RuntimeState::new(),
            source,
            sink,
            clock,
            queue: VecDeque::new(),
        })
    }

    /// Get a snapshot of the current state.
    pub fn state(&self) -> &RuntimeState {
        &self.state
    }

    /// Get a copy of the current config.
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Inject an event from external code (push-style).
    ///
    /// This does *not* immediately process the event; it is queued and
    /// consumed on the next `tick_once` call.
    pub fn inject_event(&mut self, mut event: ModuleEvent) {
        // If producer forgot to set timestamp, we populate it.
        if event.timestamp.timestamp_millis() == 0 {
            event.timestamp = Utc::now();
        }
        self.queue.push_back(event);
    }

    /// Apply a control command.
    pub fn apply_command(&mut self, cmd: RuntimeCommand) -> Result<(), RuntimeError> {
        match cmd {
            RuntimeCommand::Start => {
                match self.state.status {
                    RuntimeStatus::Initialized | RuntimeStatus::Stopped | RuntimeStatus::Failed => {
                        self.state.status = RuntimeStatus::Running;
                        self.state.tick_counter = 0;
                        self.state.last_error = None;
                        Ok(())
                    }
                    RuntimeStatus::Running | RuntimeStatus::Paused => Ok(()),
                }
            }
            RuntimeCommand::Stop => {
                self.state.status = RuntimeStatus::Stopped;
                Ok(())
            }
            RuntimeCommand::Pause => {
                if self.state.status == RuntimeStatus::Running {
                    self.state.status = RuntimeStatus::Paused;
                }
                Ok(())
            }
            RuntimeCommand::Resume => {
                if matches!(
                    self.state.status,
                    RuntimeStatus::Paused | RuntimeStatus::Initialized | RuntimeStatus::Stopped
                ) {
                    self.state.status = RuntimeStatus::Running;
                }
                Ok(())
            }
            RuntimeCommand::Step => {
                // Step is a convenience; we just do one tick without sleeping.
                self.tick_once().map(|_| ())
            }
            RuntimeCommand::Reconfigure(cfg) => {
                cfg.validate()
                    .map_err(RuntimeError::InvalidConfig)?;
                self.config = cfg;
                Ok(())
            }
        }
    }

    /// Run a single tick of the runtime.
    ///
    /// This is the core of the loop and is intentionally small and deterministic.
    pub fn tick_once(&mut self) -> Result<RuntimeTickStats, RuntimeError> {
        match self.state.status {
            RuntimeStatus::Running => {}
            RuntimeStatus::Paused => return Err(RuntimeError::Paused),
            status => return Err(RuntimeError::NotRunning(status)),
        }

        let tick_number = self.state.tick_counter + 1;

        // 1) Poll source for new events (pull-style).
        let polled_events = self
            .source
            .poll_events(self.config.max_events_per_tick)
            .map_err(|e| self.handle_error(e))?;

        let events_polled = polled_events.len();
        for ev in polled_events {
            self.queue.push_back(ev);
        }

        // 2) Process up to max_events_per_tick events from the queue.
        let mut processed = 0usize;
        let mut emitted_actions: Vec<RuntimeAction> = Vec::new();

        while processed < self.config.max_events_per_tick {
            let event = match self.queue.pop_front() {
                Some(ev) => ev,
                None => break,
            };

            // For now, runtime is policy- and threshold-agnostic.
            // It simply translates events into "noop" actions or
            // passes them through untouched when policy is wired in.
            //
            // Placeholder: we simply log-able transform the event into a
            // synthetic "debug/log" action for demonstration.
            let action = RuntimeAction {
                target: "logs".to_string(),
                kind: "runtime_debug_event".to_string(),
                parameters: serde_json::json!({
                    "source": event.source,
                    "kind": event.kind,
                    "timestamp": event.timestamp,
                }),
            };

            emitted_actions.push(action);
            processed += 1;
        }

        // 3) Submit actions to sink.
        if !emitted_actions.is_empty() {
            self.sink
                .submit_actions(&emitted_actions)
                .map_err(|e| self.handle_error(e))?;
        }

        // 4) Update tick counter and maybe flush (flush behaviour will be added later).
        self.state.tick_counter = tick_number;

        let flushed = if self.config.flush_interval_ticks > 0
            && (tick_number % self.config.flush_interval_ticks == 0)
        {
            // Placeholder: this is where we would flush logs / snapshots.
            true
        } else {
            false
        };

        let stats = RuntimeTickStats {
            tick_number,
            events_polled,
            events_processed: processed,
            actions_emitted: emitted_actions.len(),
            flushed,
        };

        // 5) Sleep until next tick.
        let sleep_duration = Duration::from_millis(self.config.tick_interval_ms);
        self.clock.sleep(sleep_duration);

        Ok(stats)
    }

    /// Internal helper: apply error policy and possibly transition to Failed.
    fn handle_error(&mut self, err: RuntimeError) -> RuntimeError {
        if self.config.fail_fast {
            self.state.record_error(&err);
        }
        err
    }
}