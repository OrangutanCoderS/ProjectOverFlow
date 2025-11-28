use std::collections::HashMap;
use std::time::Duration;

use auth_layer::{AuthLayer, AuthToken, PeerKeyStore};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{Duration as ChronoDuration, Utc};
use ed25519_dalek::{SigningKey, VerifyingKey};
use ed25519_dalek::Signer; // 🔥 REQUIRED
use rand::rngs::OsRng;     // 🔥 REQUIRED

fn build_layer_and_signer() -> (AuthLayer, SigningKey, String) {
    let mut rng = OsRng;
    let sk = SigningKey::generate(&mut rng);
    let vk: VerifyingKey = sk.verifying_key();

    let peer_id = "peer-A".to_string();
    let mut raw_map = HashMap::new();
    raw_map.insert(peer_id.clone(), vk.to_bytes().to_vec());

    let keystore = PeerKeyStore::from_bytes_map(&raw_map).expect("keystore must build");

    let layer = AuthLayer::new(keystore, 1024, Duration::from_secs(30));

    (layer, sk, peer_id)
}

fn make_token(sk: &SigningKey, peer_id: &str, payload: &[u8]) -> AuthToken {
    let timestamp = Utc::now().to_rfc3339();
    let nonce = "nonce-1".to_string();

    let sig_bytes = sk.sign(payload).to_bytes(); // now works
    let sig_b64 = STANDARD.encode(sig_bytes);

    AuthToken {
        peer_id: peer_id.to_string(),
        timestamp,
        nonce,
        signature_b64: sig_b64,
    }
}

#[test]
fn test_validate_success() {
    let (layer, sk, peer_id) = build_layer_and_signer();
    let payload = br#"{"cpu":0.10,"mem":0.20}"#;

    let token = make_token(&sk, &peer_id, payload);

    assert!(layer.validate_token(&token, payload).is_ok());
}

#[test]
fn test_replay_rejected() {
    let (layer, sk, peer_id) = build_layer_and_signer();
    let payload = br#"{"cpu":0.11,"mem":0.21}"#;

    let token = make_token(&sk, &peer_id, payload);

    assert!(layer.validate_token(&token, payload).is_ok());
    assert!(layer.validate_token(&token, payload).is_err());
}

#[test]
fn test_unknown_peer_rejected() {
    let (layer, sk, _peer_id) = build_layer_and_signer();
    let payload = br#"{"cpu":0.12,"mem":0.22}"#;

    let mut token = make_token(&sk, "peer-A", payload);
    token.peer_id = "peer-UNKNOWN".to_string();

    assert!(layer.validate_token(&token, payload).is_err());
}

#[test]
fn test_bad_signature_rejected() {
    let (layer, sk, peer_id) = build_layer_and_signer();
    let payload = br#"{"cpu":0.13,"mem":0.23}"#;

    let mut token = make_token(&sk, &peer_id, payload);

    let mut bytes = token.signature_b64.into_bytes();
    bytes[5] ^= 0x5A;
    token.signature_b64 = String::from_utf8(bytes).unwrap();

    assert!(layer.validate_token(&token, payload).is_err());
}

#[test]
fn test_clock_skew_rejected() {
    let (layer, sk, peer_id) = build_layer_and_signer();
    let payload = br#"{"cpu":0.14,"mem":0.24}"#;

    let mut token = make_token(&sk, &peer_id, payload);
    let old_ts = Utc::now() - ChronoDuration::seconds(7200);
    token.timestamp = old_ts.to_rfc3339();

    assert!(layer.validate_token(&token, payload).is_err());
}

#[test]
fn test_public_keystore_rejects_bad_length() {
    use auth_layer::error::AuthError;

    let mut raw_map = HashMap::new();
    raw_map.insert("peer-X".to_string(), vec![1, 2, 3]);

    let result = PeerKeyStore::from_bytes_map(&raw_map);
    assert!(matches!(result, Err(AuthError::BadPublicKeyLen(3))));
}