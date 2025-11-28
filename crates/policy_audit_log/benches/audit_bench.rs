use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;
use tempfile::tempdir;

use policy_audit_log::{AuditEvent, AuditEventKind, AuditLogWriter};

fn bench_audit_append(c: &mut Criterion) {
    c.bench_function("audit_append_1000_events", |b| {
        b.iter(|| {
            let dir = tempdir().unwrap();
            let path = dir.path().join("audit.log");
            let writer = AuditLogWriter::new(&path).unwrap();

            for i in 0..1000u32 {
                let ev = AuditEvent {
                    timestamp: Utc::now(),
                    actor: "bench".to_string(),
                    policy_id: format!("policy-{}", i),
                    kind: AuditEventKind::PolicyUpdated,
                    details: json!({ "k": "v", "seq": i }),
                };
                writer.append_event(black_box(ev)).unwrap();
            }

            writer.sync().unwrap();
        });
    });
}

criterion_group!(benches, bench_audit_append);
criterion_main!(benches);
