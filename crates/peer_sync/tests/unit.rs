use peer_sync::{
    signer::PeerSigner,
    verifier::PeerVerifier,
    model::SyncMessage,
};
use chrono::Utc;
    use serde_json::json;

// Helper: make a raw message (Vec<u8>)
fn make_raw_message() -> Vec<u8> {
    let msg = SyncMessage {
        origin_id: "peer-A".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        payload: json!({"cpu": 0.77, "mem": 0.33}),
        signature: String::new(),
    };
    msg.raw_bytes().to_vec()
}

#[test]
fn test_sign_and_verify_success() {
    let signer = PeerSigner::generate();
    let raw = make_raw_message();
    let sig = signer.sign(&raw);

    let pk = signer.public_key_bytes();
    let verifier = PeerVerifier::new(&pk).unwrap();

    assert!(verifier.verify(&raw, &sig).is_ok());
}

#[test]
fn test_verify_fails_on_tampered_payload() {
    let signer = PeerSigner::generate();
    let raw = make_raw_message();
    let sig = signer.sign(&raw);

    let pk = signer.public_key_bytes();
    let verifier = PeerVerifier::new(&pk).unwrap();

    // Tamper ONE byte
    let mut tampered = raw.clone();
    tampered[5] ^= 0xAA;

    assert!(verifier.verify(&tampered, &sig).is_err());
}

#[test]
fn test_verify_fails_with_wrong_public_key() {
    let signer1 = PeerSigner::generate();
    let signer2 = PeerSigner::generate();

    let raw = make_raw_message();
    let sig = signer1.sign(&raw);

    let wrong_pk = signer2.public_key_bytes();
    let verifier = PeerVerifier::new(&wrong_pk).unwrap();

    assert!(verifier.verify(&raw, &sig).is_err());
}

#[test]
fn test_bad_signature_length() {
    let signer = PeerSigner::generate();
    let raw = make_raw_message();
    let sig = signer.sign(&raw);

    let pk = signer.public_key_bytes();
    let verifier = PeerVerifier::new(&pk).unwrap();

    // Chop signature to invalid length
    let mut short_sig = sig.clone();
    short_sig.truncate(10);

    assert!(verifier.verify(&raw, &short_sig).is_err());
}

#[test]
fn test_bad_public_key_length() {
    let bad_key = vec![1, 2, 3]; // only 3 bytes, invalid
    let result = PeerVerifier::new(&bad_key);
    assert!(result.is_err());
}

#[test]
fn test_multiple_signatures_stable() {
    let signer = PeerSigner::generate();
    let raw = make_raw_message();

    let sig1 = signer.sign(&raw);
    let sig2 = signer.sign(&raw);

    // Ed25519 signatures MUST be deterministic under dalek v2
    assert_eq!(sig1, sig2);
}

#[test]
fn test_verifier_rejects_modified_signature() {
    let signer = PeerSigner::generate();
    let raw = make_raw_message();
    let sig = signer.sign(&raw);

    // Convert signature → bytes → mutate → back to string
    let mut sig_bytes = sig.into_bytes();
    sig_bytes[3] ^= 0x5A;
    let tampered_sig = String::from_utf8(sig_bytes).unwrap();

    let pk = signer.public_key_bytes();
    let verifier = PeerVerifier::new(&pk).unwrap();

    assert!(verifier.verify(&raw, &tampered_sig).is_err());
}