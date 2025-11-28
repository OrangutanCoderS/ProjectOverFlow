//! Action Kill module — deterministic and auditable process termination layer.
//! Safe across macOS + Linux. Designed for controlled enforcement, not brute kill.

#![deny(unsafe_op_in_unsafe_fn)]
#![warn(clippy::all, clippy::pedantic)]
#![deny(unsafe_code)] // ⬅️ changed from forbid to deny (still fails everywhere else)

pub mod manager;
pub mod policy;
pub mod request;
pub mod audit;
pub mod errors;
pub mod controller;

pub use manager::ActionKillManager;
pub use policy::{KillPolicy, PolicyLoader};
pub use request::{KillRequest, KillMode, KillResult};
pub use audit::AuditLogger;
pub use errors::ManagerError;