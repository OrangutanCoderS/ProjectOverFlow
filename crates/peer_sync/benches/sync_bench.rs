use criterion::{criterion_group, criterion_main, Criterion, black_box};
use peer_sync::{
    signer::PeerSigner,
    verifier::PeerVerifier,
    model::SyncMessage,
    packet::WirePacket
};
use chrono::Utc;

/// Generate a sample SyncMessage
fn make_message() -> SyncMessage {
    SyncMessage {
        origin_id: "bench-node".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        payload: serde_json::json!({
            "cpu": 0.5,
            "mem": 0.2,
            "load": 1.25
        }),
        signature: String::new(),
    }
}

fn bench_signing(c: &mut Criterion) {
    let signer = PeerSigner::generate();
    let msg = make_message();
    let raw = msg.raw_bytes();

    c.bench_function("sign_message", |b| {
        b.iter(|| {
            let sig = signer.sign(raw.as_ref());
            black_box(sig);
        })
    });
}

fn bench_verifying(c: &mut Criterion) {
    let signer = PeerSigner::generate();
    let mut msg = make_message();
    let raw = msg.raw_bytes();
    msg.signature = signer.sign(raw.as_ref());

    let pubkey = signer.public_key_bytes();
    let verifier = PeerVerifier::new(&pubkey).unwrap();

    c.bench_function("verify_message", |b| {
        b.iter(|| {
            let _ = verifier.verify(raw.as_ref(), &msg.signature);
        })
    });
}

fn bench_serialization(c: &mut Criterion) {
    let msg = make_message();
    let pkt = WirePacket { msg };

    c.bench_function("json_serialize", |b| {
        b.iter(|| {
            let out = serde_json::to_vec(&pkt).unwrap();
            black_box(out);
        })
    });

    let buf = serde_json::to_vec(&pkt).unwrap();

    c.bench_function("json_deserialize", |b| {
        b.iter(|| {
            let parsed: WirePacket = serde_json::from_slice(&buf).unwrap();
            black_box(parsed);
        })
    });
}

criterion_group!(
    benches,
    bench_signing,
    bench_verifying,
    bench_serialization
);
criterion_main!(benches);
