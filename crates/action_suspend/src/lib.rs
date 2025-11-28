#![deny(unsafe_op_in_unsafe_fn)]
// Modules
pub mod manager;
pub mod controller;
pub mod audit;
pub mod errors;
pub mod request;

// Re-exports for crate-wide access
pub use crate::manager::ActionSuspendManager;
pub use crate::errors::ManagerError;
pub use crate::request::{SuspendMode, SuspendRequest};

// Optional: a smoke test hook for `cargo test`
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}