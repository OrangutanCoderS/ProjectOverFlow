use criterion::{black_box, criterion_group, criterion_main, Criterion};
use export_sanitizer::{ExportSanitizer, Salt, SanitizerRules};
use serde_json::json;

fn build_sanitizer() -> ExportSanitizer {
    let rules = SanitizerRules::default();
    let salt = Salt::random();
    ExportSanitizer::new(rules, salt)
}

fn bench_sanitizer(c: &mut Criterion) {
    let sanitizer = build_sanitizer();

    let mut packet = json!({
        "meta": {
            "trusted": false,
            "env": "prod"
        },
        "host": "my-sensitive-host.internal.company.com",
        "payload": {
            "paths": [
                "/home/alice/.ssh/id_rsa",
                "/home/bob/Documents/secret.docx",
                "C:\\Users\\carol\\Downloads\\private.pdf"
            ],
            "network": {
                "ip": "192.168.1.100",
                "mac": "de:ad:be:ef:00:01",
                "log": "192.168.1.100 connected as de:ad:be:ef:00:01"
            }
        }
    });

    c.bench_function("export_sanitizer_sanitize_packet", |b| {
        b.iter(|| {
            let mut pkt = packet.clone();
            let report = sanitizer.sanitize_packet(&mut pkt);
            black_box(pkt);
            black_box(report.total_redactions());
        });
    });
}

criterion_group!(benches, bench_sanitizer);
criterion_main!(benches);
