use std::time::Duration;

use socket_listener::{rate_limit::RateLimiter, replay::ReplayProtection, model::MeshPacket};

#[test]
fn rate_limiter_blocks_after_threshold() {
    use std::net::IpAddr;

    let rl = RateLimiter::new(Duration::from_secs(1), 3);
    let ip: IpAddr = "127.0.0.1".parse().unwrap();

    assert_eq!(rl.is_limited(&ip), false);
    assert_eq!(rl.is_limited(&ip), false);
    assert_eq!(rl.is_limited(&ip), false);
    assert_eq!(rl.is_limited(&ip), true);
}

#[test]
fn replay_protection_rejects_replays_and_skew() {
    use chrono::{Duration as ChronoDuration, Utc};

    let rp = ReplayProtection::new(1024, Duration::from_secs(60));

    let now = Utc::now();
    let ts = now.to_rfc3339();

    assert_eq!(rp.validate("node-a", &ts), true);
    assert_eq!(rp.validate("node-a", &ts), false);

    let old_ts = (now - ChronoDuration::seconds(3600)).to_rfc3339();
    assert_eq!(rp.validate("node-a", &old_ts), false);
}

#[test]
fn mesh_packet_parses_and_validates() {
    let raw = r#"
    {
        "packet_type": "TELEMETRY_PACKET",
        "origin_id": "node-123",
        "timestamp": "2025-01-01T00:00:00Z",
        "payload": {"cpu": 0.75},
        "auth_token": "secret"
    }
    "#;

    let pkt: MeshPacket = serde_json::from_str(raw).expect("valid JSON");
    assert!(pkt.is_structurally_valid());
    assert_eq!(pkt.packet_kind().is_some(), true);
}
