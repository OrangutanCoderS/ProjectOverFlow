use chrono::Utc;
use log::warn;
use std::fs::OpenOptions;
use std::io::Write;

pub fn record_event(metric: &str, value: f64, decision: &str) {
    let timestamp = Utc::now().to_rfc3339();
    let log_line = format!("{timestamp} | {metric}={value:.2} -> {decision}\n");

    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/threshold_audit.log")
    {
        if let Err(e) = f.write_all(log_line.as_bytes()) {
            warn!("Audit log write failed: {}", e);
        }
    }
}