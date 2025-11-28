//! memory_patcher — Phase II Module 4
//! Safe, verifiable memory scanning and patching with mock backend by default.

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
mod scanner;
mod patcher;
pub mod sys;

use error::PatchError;
pub use patcher::{PatchSpecV1, PatchResult, list_memory_regions, find_signatures, apply_patch};
pub use sys::MemoryRegion;
pub use scanner::Signature;
use sha2::{Digest, Sha256};

/// Utility: compute sha256 hex (helpful for tests and logs)
pub fn sha256_hex(b: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(b);
    let out = h.finalize();
    out.iter().map(|v| format!("{:02x}", v)).collect::<String>()
}