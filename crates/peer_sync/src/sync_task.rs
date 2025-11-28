use tokio::time::{sleep, Duration};
use crate::{signer::PeerSigner, dispatcher::Dispatcher, model::SyncMessage};
use chrono::Utc;

/// Periodic peer sync loop.
pub async fn run_sync_loop(peer_addr: String) {
    let signer = PeerSigner::generate();

    loop {
        let payload = serde_json::json!({
            "cpu": 0.75,
            "mem": 0.42
        });

        let timestamp = Utc::now().to_rfc3339();
        let mut msg = SyncMessage {
            origin_id: "overlow-node".to_string(),
            timestamp: timestamp.clone(),
            payload,
            signature: String::new(),
        };

        let raw = msg.raw_bytes();
        msg.signature = signer.sign(&raw);

        let _ = Dispatcher::send(&peer_addr, msg).await;

        sleep(Duration::from_secs(2)).await;
    }
}
