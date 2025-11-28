use criterion::{criterion_group, criterion_main, Criterion, black_box};
use socket_listener::model::{MeshPacket, PacketType};
use serde_json::Value;

fn sample_packet() -> &'static str {
    r#"
    {
        "packet_type": "TELEMETRY_PACKET",
        "origin_id": "node-123",
        "timestamp": "2025-01-01T00:00:00Z",
        "payload": {"cpu": 0.75, "mem": 0.42},
        "auth_token": "secret"
    }
    "#
}

/// Benchmark 1: full MeshPacket deserialization
fn bench_meshpacket_parse(c: &mut Criterion) {
    let raw = sample_packet();

    c.bench_function("mesh_packet_parse/full_struct", |b| {
        b.iter(|| {
            let pkt: MeshPacket = serde_json::from_str(raw).unwrap();
            black_box(pkt);
        });
    });
}

/// Benchmark 2: raw JSON only
fn bench_raw_json_parse(c: &mut Criterion) {
    let raw = sample_packet();

    c.bench_function("mesh_packet_parse/raw_json_only", |b| {
        b.iter(|| {
            let val: Value = serde_json::from_str(raw).unwrap();
            black_box(val);
        });
    });
}

/// Benchmark 3: preallocated string parse
fn bench_preallocated_parsing(c: &mut Criterion) {
    let raw = sample_packet();
    let mut s = String::with_capacity(raw.len());
    s.push_str(raw);

    c.bench_function("mesh_packet_parse/preallocated_string", |b| {
        b.iter(|| {
            let pkt: MeshPacket = serde_json::from_str(&s).unwrap();
            black_box(pkt);
        });
    });
}

/// Benchmark 4: enum-validation benchmark
fn bench_validation_logic(c: &mut Criterion) {
    c.bench_function("mesh_packet_validate/enum_match", |b| {
        b.iter(|| {
            let p = PacketType::Telemetry;

            let res = match p {
                PacketType::Telemetry => true,
                PacketType::PluginGraphUpdate => false,
                PacketType::SecurityAlert => false,
            };

            black_box(res);
        });
    });
}

/// Benchmark group
criterion_group!(
    benches,
    bench_meshpacket_parse,
    bench_raw_json_parse,
    bench_preallocated_parsing,
    bench_validation_logic
);
criterion_main!(benches);