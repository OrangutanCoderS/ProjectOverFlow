use chrono::Utc;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Serialize)]
pub struct FakeOutputEvent<'a> {
    timestamp: String,
    pid: Option<i32>,
    action: &'a str,
    target: &'a str,
    fake_payload: Option<&'a str>,
    result: &'a str,
}

pub fn log_event(pid: Option<i32>, action: &str, target: &str, payload: Option<&str>, result: &str) {
    let event = FakeOutputEvent {
        timestamp: Utc::now().to_rfc3339(),
        pid,
        action,
        target,
        fake_payload: payload,
        result,
    };

    if let Ok(json) = serde_json::to_string(&event) {
        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open("logs/system_log.json")
            .and_then(|mut f| writeln!(f, "{}", json));
    }
}