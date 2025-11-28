#![forbid(unsafe_code)]
use super::{MemoryRegion, OsProcessMem};
use crate::error::PatchError;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ops::Range;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct MockProc {
    regions: Vec<(String, Vec<u8>, String)>, // (name, bytes, prot)
    base_addrs: Vec<usize>,                   // start addrs for each region
}

static MOCK_DB: Lazy<RwLock<HashMap<i32, MockProc>>> = Lazy::new(|| {
    let mut db = HashMap::new();
    // PID 1234 with two regions (deterministic data)
    db.insert(1234, MockProc {
        regions: vec![
            (".text".into(), vec![0x48, 0x8B, 0x05, 0x90, 0x90, 0x55, 0x8B], "r-x".into()),
            (".data".into(), b"OverFlow\x00\xFF\xEE".to_vec(), "rw-".into())
        ],
        base_addrs: vec![0x1000_0000, 0x2000_0000],
    });
    RwLock::new(db)
});

pub struct MockBackend;

impl MockBackend {
    pub fn new() -> Self { Self }
}

impl OsProcessMem for MockBackend {
    fn check_permission(&self, pid: i32) -> Result<(), PatchError> {
        let db = MOCK_DB.read();
        if db.contains_key(&pid) { Ok(()) } else { Err(PatchError::ProcessGone) }
    }

    fn list_regions(&self, pid: i32) -> Result<Vec<MemoryRegion>, PatchError> {
        let db = MOCK_DB.read();
        let p = db.get(&pid).ok_or(PatchError::ProcessGone)?;
        Ok(p.regions.iter().enumerate().map(|(i, (name, bytes, prot))| {
            MemoryRegion {
                name: name.clone(),
                range: p.base_addrs[i]..(p.base_addrs[i] + bytes.len()),
                prot: prot.clone(),
            }
        }).collect())
    }

    fn read(&self, pid: i32, addr: usize, buf: &mut [u8], timeout: Duration) -> Result<(), PatchError> {
        deadline(timeout)?;
        let db = MOCK_DB.read();
        let p = db.get(&pid).ok_or(PatchError::ProcessGone)?;
        let (region_idx, off) = locate(&p, addr, buf.len()).ok_or(PatchError::Validation("read out of range".into()))?;
        buf.copy_from_slice(&p.regions[region_idx].1[off..off + buf.len()]);
        Ok(())
    }

    fn write(&self, pid: i32, addr: usize, data: &[u8], timeout: Duration) -> Result<(), PatchError> {
        deadline(timeout)?;
        let mut db = MOCK_DB.write();
        let p = db.get_mut(&pid).ok_or(PatchError::ProcessGone)?;
        let (region_idx, off) = locate(p, addr, data.len()).ok_or(PatchError::Validation("write out of range".into()))?;
        if p.regions[region_idx].2 != "rw-" {
            return Err(PatchError::PermissionDenied("region not writable".into()));
        }
        p.regions[region_idx].1[off..off + data.len()].copy_from_slice(data);
        Ok(())
    }
}

fn locate(p: &MockProc, addr: usize, len: usize) -> Option<(usize, usize)> {
    for (i, base) in p.base_addrs.iter().enumerate() {
        let end = base + p.regions[i].1.len();
        if addr >= *base && (addr + len) <= end {
            let off = addr - *base;
            return Some((i, off));
        }
    }
    None
}

fn deadline(timeout: Duration) -> Result<(), PatchError> {
    // Simulate time check (instantaneous pass) — placeholder for future
    let _t0 = Instant::now();
    if timeout.is_zero() { /* treat as no-timeout */ }
    Ok(())
}