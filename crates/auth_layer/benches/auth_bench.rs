use std::collections::HashMap;
use std::time::Duration;

use auth_layer::{AuthLayer, AuthToken, PeerKeyStore};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ed25519_dalek::{SigningKey, VerifyingKey};
use ed25519_dalek::Signer;            // 🔥 REQUIRED
use rand::rngs::OsRng;                // 🔥 REQUIRED

fn build_layer() -> (AuthLayer, SigningKey, String) {
    let mut rng = OsRng;
    let sk = SigningKey::generate(&mut rng);
    let vk: VerifyingKey = sk.verifying_key();

    let peer_id = "peer-A".to_string();
    let mut raw_map = HashMap::new();
    raw_map.insert(peer_id.clone(), vk.to_bytes().to_vec());

    let keystore = PeerKeyStore::from_bytes_map(&raw_map).unwrap();

    let layer = AuthLayer::new(keystore, 1024, Duration::from_secs(30));

    (layer, sk, peer_id)
}

fn bench_auth_validate(c: &mut Criterion) {
    let (layer, sk, peer_id) = build_layer();
    let payload = br#"{"cpu":0.42,"mem":0.73}"#;

    c.bench_function("auth_validate_token", |b| {
        b.iter(|| {
            let nonce = format!("n-{}", black_box(Utc::now().timestamp_nanos()));

            let timestamp = Utc::now().to_rfc3339();
            let sig_bytes = sk.sign(payload).to_bytes(); // works now
            let sig_b64 = STANDARD.encode(sig_bytes);

            let token = AuthToken {
                peer_id: peer_id.clone(),
                timestamp,
                nonce,
                signature_b64: sig_b64,
            };

            layer.validate_token(&token, payload).unwrap();

            black_box(());
        });
    });
}

criterion_group!(benches, bench_auth_validate);
criterion_main!(benches);