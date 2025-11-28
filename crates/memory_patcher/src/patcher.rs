#![forbid(unsafe_code)]
use crate::error::PatchError;
use crate::scanner::{Signature, find_all};
use crate::sys::{backend, MemoryRegion, OsProcessMem};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use sha2::{Sha256, Digest};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct PatchSpecV1 {
    pub pid: i32,
    pub signature: Signature,   // required
    pub offset: isize,          // may be zero
    pub new_bytes: Vec<u8>,     // required
    pub region_hint: Option<String>,
    pub checksum_before: Option<Vec<u8>>, // raw sha256 bytes
    pub checksum_after: Option<Vec<u8>>,  // raw sha256 bytes
    pub dry_run: bool,
    pub timeout: Duration,
    pub rollback_on_fail: bool,
}

#[derive(Debug, Clone)]
pub struct FoundMatch {
    pub region: MemoryRegion,
    pub addr: usize,        // absolute address of match start
}

#[derive(Debug, Clone)]
pub struct PatchResult {
    pub addr: usize,
    pub before_sha256: Vec<u8>,
    pub after_sha256: Option<Vec<u8>>, // None on dry-run
    pub bytes_len: usize,
    pub dry_run: bool,
}

pub fn decode_base64_bytes(s: &str) -> Result<Vec<u8>, PatchError> {
    STANDARD.decode(s).map_err(|e| PatchError::Validation(format!("bad base64: {e}")))
}

fn sha256(data: &[u8]) -> Vec<u8> {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().to_vec()
}

pub fn list_memory_regions(pid: i32) -> Result<Vec<MemoryRegion>, PatchError> {
    let be = backend();
    be.check_permission(pid)?;
    be.list_regions(pid)
}

pub fn find_signatures(pid: i32, sig: &Signature, region_hint: Option<&str>, timeout: Duration)
    -> Result<Vec<FoundMatch>, PatchError>
{
    let be = backend();
    be.check_permission(pid)?;
    let regions = be.list_regions(pid)?;

    let mut out = Vec::new();
    for reg in regions.iter() {
        if let Some(hint) = region_hint {
            if !reg.name.contains(hint) { continue; }
        }
        let len = reg.range.end - reg.range.start;
        let mut buf = vec![0u8; len];
        be.read(pid, reg.range.start, &mut buf, timeout)?;
        for off in find_all(&buf, sig) {
            out.push(FoundMatch { region: reg.clone(), addr: reg.range.start + off });
        }
    }
    if out.is_empty() { return Err(PatchError::NotFound); }
    Ok(out)
}

pub fn apply_patch(spec: &PatchSpecV1) -> Result<PatchResult, PatchError> {
    if spec.new_bytes.is_empty() { return Err(PatchError::Validation("new_bytes empty".into())); }
    if spec.offset.abs() as usize > 1_048_576 { // guard absurd offsets
        return Err(PatchError::Validation("offset too large".into()));
    }

    let be = backend();
    be.check_permission(spec.pid)?;

    // Find match
    let hits = find_signatures(spec.pid, &spec.signature, spec.region_hint.as_deref(), spec.timeout)?;
    // v1: first match wins (future v2 may allow choose Nth / all)
    let first = &hits[0];
    let target_addr = if spec.offset >= 0 {
        first.addr + (spec.offset as usize)
    } else {
        first.addr.checked_sub((-spec.offset) as usize)
            .ok_or_else(|| PatchError::Validation("offset underflow".into()))?
    };

    // read-before
    let mut before = vec![0u8; spec.new_bytes.len()];
    be.read(spec.pid, target_addr, &mut before, spec.timeout)?;
    let before_sha = sha256(&before);

    // verify checksum_before if provided
    if let Some(exp) = &spec.checksum_before {
        if *exp != before_sha { return Err(PatchError::ChecksumMismatch); }
    }

    if spec.dry_run {
        return Ok(PatchResult {
            addr: target_addr,
            before_sha256: before_sha,
            after_sha256: None,
            bytes_len: spec.new_bytes.len(),
            dry_run: true,
        });
    }

    // write new bytes
    be.write(spec.pid, target_addr, &spec.new_bytes, spec.timeout)?;

    // read-after + verify
    let mut after = vec![0u8; spec.new_bytes.len()];
    be.read(spec.pid, target_addr, &mut after, spec.timeout)?;
    let after_sha = sha256(&after);

    if let Some(exp) = &spec.checksum_after {
        if *exp != after_sha {
            // rollback
            if spec.rollback_on_fail {
                let _ = be.write(spec.pid, target_addr, &before, spec.timeout);
                return Err(PatchError::ChecksumMismatch);
            }
        }
    }

    Ok(PatchResult {
        addr: target_addr,
        before_sha256: before_sha,
        after_sha256: Some(after_sha),
        bytes_len: spec.new_bytes.len(),
        dry_run: false,
    })
}