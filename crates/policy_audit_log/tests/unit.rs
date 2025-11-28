use std::fs;
use std::path::PathBuf;

use policy_audit_log::{
    model::{AuditEvent, AuditEventKind},
    AuditLogReader,
    AuditLogWriter,
};

fn make_event(payload: &str) -> AuditEvent {
    AuditEvent {
        kind: AuditEventKind::PolicyApplied,
        payload: payload.to_string(),
    }
}

#[test]
fn append_and_roundtrip_works() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test_roundtrip.log");
    let _ = fs::remove_file(&path);

    let mut writer = AuditLogWriter::new(&path)?;
    writer.append(make_event("p-1"))?;
    writer.append(make_event("p-2"))?;
    writer.append(make_event("p-3"))?;

    let entries = AuditLogReader::load(&path)?;
    assert_eq!(entries.len(), 3);

    assert_eq!(entries[0].event.payload, "p-1");
    assert_eq!(entries[1].event.payload, "p-2");
    assert_eq!(entries[2].event.payload, "p-3");

    Ok(())
}

#[test]
fn detects_chain_tampering() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test_tamper.log");
    let _ = fs::remove_file(&path);

    let mut writer = AuditLogWriter::new(&path)?;
    writer.append(make_event("good-1"))?;
    writer.append(make_event("good-2"))?;

    // tamper with the file directly
    fs::write(&path, b"{not: valid json}\n")?;

    let result = AuditLogReader::load(&path);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn detects_index_inconsistency() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test_index_err.log");
    let _ = fs::remove_file(&path);

    let mut writer = AuditLogWriter::new(&path)?;
    writer.append(make_event("A"))?;
    writer.append(make_event("B"))?;

    // manually mess index
    let mut contents = fs::read_to_string(&path)?;
    contents = contents.replace("\"index\":1", "\"index\":5");
    fs::write(&path, contents)?;

    let result = AuditLogReader::load(&path);
    assert!(result.is_err());
    Ok(())
}