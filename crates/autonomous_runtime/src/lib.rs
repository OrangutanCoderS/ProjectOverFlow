pub mod config;
pub mod error;
pub mod state;
pub mod event;
pub mod source;
pub mod sink;
pub mod runtime;
pub mod clock;

pub use crate::config::RuntimeConfig;
pub use crate::error::RuntimeError;
pub use crate::state::{RuntimeState, RuntimeStatus};
pub use crate::event::{ModuleEvent, RuntimeAction};
pub use crate::runtime::{AutonomousRuntime, RuntimeCommand, RuntimeTickStats};
pub use crate::clock::{RuntimeClock, SystemClock};
pub use crate::clock::NoopClock;

// FIX: export these so tests can import them from the crate root
pub use crate::sink::{ActionSink, NullActionSink};
pub use crate::source::{EventSource, NullEventSource};