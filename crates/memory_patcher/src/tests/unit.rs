#![forbid(unsafe_code)]
use memory_patcher::{
    Signature, list_memory_regions, find_signatures, PatchSpecV1, apply_patch, sha256_hex
};
use std::time::Duration;

#[test]
fn list_regions_ok() {
    let regs = list_memory_regions(1234).expect("mock pid exists");
    assert!(!regs.is_empty());
    assert!(regs.iter().any(|r| r.name == ".text"));
    assert!(regs.iter().any(|r| r.name == ".data"));
}

#[test]
fn find_sig_and_dry_run() {
    let sig = Signature::parse("48 8B 05 ?? ?? 55 8B").unwrap();
    let hits = find_signatures(1234, &sig, Some(".text"), Duration::from_millis(50)).unwrap();
    assert!(!hits.is_empty());

    let spec = PatchSpecV1 {
        pid: 1234,
        signature: sig,
        offset: 0,
        new_bytes: vec![0x90, 0x90, 0x90], // NOP-like replacement
        region_hint: Some(".text".into()),
        checksum_before: None,
        checksum_after: None,
        dry_run: true,
        timeout: Duration::from_millis(50),
        rollback_on_fail: true,
    };
    let res = apply_patch(&spec).unwrap();
    assert!(res.dry_run);
    assert_eq!(res.bytes_len, 3);
    assert!(!res.before_sha256.is_empty());
}

#[test]
fn live_patch_and_verify() {
    // Patch inside the .data region where writes are allowed
    let sig = Signature::parse("4F 76 65 72 46 6C 6F 77 00 FF EE").unwrap(); // "OverFlow\x00\xFF\xEE"
    let hits = find_signatures(1234, &sig, Some(".data"), Duration::from_millis(50)).unwrap();
    let addr = hits[0].addr;

    // Expect to modify first 2 bytes
    let spec = PatchSpecV1 {
        pid: 1234,
        signature: Signature::parse("4F 76").unwrap(),
        offset: 0,
        new_bytes: vec![0x41, 0x41], // "AA"
        region_hint: Some(".data".into()),
        checksum_before: None,
        checksum_after: None,
        dry_run: false,
        timeout: Duration::from_millis(50),
        rollback_on_fail: true,
    };
    let res = apply_patch(&spec).unwrap();
    assert!(!res.dry_run);
    assert_eq!(res.bytes_len, 2);

    // Confirm the first two bytes changed by re-reading via another dry-run compare:
    // (We re-use apply_patch read-before step by pointing at same address with zero-length change)
    // Instead, simpler: just ensure after SHA not equal to before SHA.
    let before_hex = sha256_hex(&res.before_sha256);
    let after_hex  = sha256_hex(&res.after_sha256.clone().unwrap());
    assert_ne!(before_hex, after_hex, "patch must change bytes");
}