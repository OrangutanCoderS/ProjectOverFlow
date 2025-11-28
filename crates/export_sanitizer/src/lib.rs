//! export_sanitizer
//!
//! Defensive data hygiene layer before any telemetry / logs leave OverFlow.
//! Scrubs or hashes paths, usernames, IPs, MACs, hostnames, with configurable
//! whitelists and paranoid mode.

mod error;
mod rules;
mod salt;
mod sanitizer;

pub use crate::error::ExportSanitizerError;
pub use crate::rules::SanitizerRules;
pub use crate::salt::Salt;
pub use crate::sanitizer::{
    ExportSanitizer, RedactionEvent, RedactionKind, SanitizationReport,
};
