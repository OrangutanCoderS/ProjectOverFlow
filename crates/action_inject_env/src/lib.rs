#![forbid(unsafe_code)]

pub mod audit;
pub mod controller;
pub mod errors;
pub mod manager;
pub mod request;

// Public re-exports for ergonomic use
pub use crate::errors::{ControllerError, ManagerError};
pub use crate::manager::ActionInjectEnvManager;
pub use crate::request::{EnvInjectionMode, InjectRequest};