use memory_patcher::{Signature, PatchSpecV1, apply_patch};
use std::time::Duration;

fn main() {
    let sig = Signature::parse("4F 76 65 72 46 6C 6F 77 00 FF EE").unwrap();
    let spec = PatchSpecV1 {
        pid: 1234,
        signature: sig,
        offset: 0,
        new_bytes: vec![0x4F, 0x76], // "Ov" (no-op for demo)
        region_hint: Some(".data".into()),
        checksum_before: None,
        checksum_after: None,
        dry_run: true,
        timeout: Duration::from_millis(50),
        rollback_on_fail: true,
    };
    let res = apply_patch(&spec).unwrap();
    println!("dry-run patch @0x{:x}, len={}, before_sha256={:x?}",
             res.addr, res.bytes_len, res.before_sha256);
}