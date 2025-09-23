//! Module 25 — Export Tools
//! CSV + JSON exporters with redaction and snapshot summaries.
//! - Safe-by-default redaction (case-insensitive by key)
//! - CSV header union across rows, stable key order
//! - Snapshot (total / first / last timestamps)

pub mod formats;
pub mod filters;

pub use formats::{csv, json, snapshot};
pub use filters::redact;

use thiserror::Error;

/// Unified error surface for all export operations.
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] ::csv::Error),  // ✅ force external crate, not local module

    #[error("Invalid input: {0}")]
    Invalid(&'static str),

    #[error("Invalid config: {0}")]
    InvalidConfig(&'static str),
}