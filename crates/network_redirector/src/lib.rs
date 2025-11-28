//! network_redirector crate
//! Safe, testable abstraction for network redirection / blocking rules.
//! Default backend: Mock (no privileges required).

pub mod model;
pub mod errors;
pub mod trait_backend;
pub mod manager;
pub mod mock_backend;
pub mod logger;
pub mod config;

#[cfg(feature = "pfctl")]
pub mod pfctl_backend;

#[cfg(feature = "iptables")]
pub mod iptables_backend;

pub use model::*;
pub use errors::*;
pub use trait_backend::*;
pub use manager::*;
pub use mock_backend::MockBackend;
pub use logger::*;
pub use config::*;