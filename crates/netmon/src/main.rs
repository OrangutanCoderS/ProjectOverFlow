use netmon::{NetworkConfig, spawn_polling};
use serde::Serialize;
use std::{env, time::{Duration, SystemTime, UNIX_EPOCH}};

#[derive(Serialize)]
struct NetworkEventEnvelope<'a> {
    event_type: &'static str,
    ts_ms: u128,
    data: &'a netmon::NetworkConnectionInfo,
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let interval = args.get(1)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1000);

    let cfg = NetworkConfig {
        interval_ms: interval,
        ..Default::default()
    };

    // backend init message (optional)
    eprintln!("ShadowTrace Network Monitor backend starting…");
    eprintln!("Backend in use: {}", netmon::backend_name());

    let rx = spawn_polling(cfg);

    loop {
        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(conn) => {
                let evt = NetworkEventEnvelope {
                    event_type: "network",
                    ts_ms: now_ms(),
                    data: &conn,
                };
                println!("{}", serde_json::to_string(&evt).unwrap());
            }
            Err(_) => {
                // Emit heartbeat to trigger UI "no data" state
                let evt = serde_json::json!({
                    "event_type": "network",
                    "ts_ms": now_ms(),
                    "data": {
                        "heartbeat": true
                    }
                });
                println!("{}", evt.to_string());
            }
        }
    }
}