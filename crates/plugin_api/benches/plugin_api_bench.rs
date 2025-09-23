use criterion::{criterion_group, criterion_main, Criterion, BatchSize};
use plugin_api::{set_config, ApiConfig, log_event, emit_alert_text};
use std::time::Duration;
use std::fs;
use chrono::Utc;

fn bench_log_event(c: &mut Criterion) {
    // Use a temp file so we don’t spam repo logs
    let tmp = tempfile_path();
    set_config(ApiConfig {
        log_path: Some(tmp.clone()),
        window: Duration::from_secs(3600),
        max_alerts_per_window: u32::MAX,
        max_logs_per_window: u32::MAX,
    });

    c.bench_function("log_event", |b| {
        b.iter_batched(
            || format!("action_{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)),
            |action| {
                log_event("bench_plugin", &action).unwrap();
            },
            BatchSize::SmallInput,
        )
    });

    fs::remove_file(tmp).ok();
}

fn bench_emit_alert(c: &mut Criterion) {
    let tmp = tempfile_path();
    set_config(ApiConfig {
        log_path: Some(tmp.clone()),
        window: Duration::from_secs(3600),
        max_alerts_per_window: u32::MAX,
        max_logs_per_window: u32::MAX,
    });

    c.bench_function("emit_alert_text", |b| {
        b.iter_batched(
            || format!("msg_{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)),
            |msg| {
                emit_alert_text("bench_plugin", "reason", &msg).unwrap();
            },
            BatchSize::SmallInput,
        )
    });

    fs::remove_file(tmp).ok();
}

fn bench_combined(c: &mut Criterion) {
    let tmp = tempfile_path();
    set_config(ApiConfig {
        log_path: Some(tmp.clone()),
        window: Duration::from_secs(3600),
        max_alerts_per_window: u32::MAX,
        max_logs_per_window: u32::MAX,
    });

    c.bench_function("log+alert", |b| {
        b.iter_batched(
            || {
                (
                    format!("act_{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)),
                    format!("msg_{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)),
                )
            },
            |(a, m)| {
                log_event("bench_plugin", &a).unwrap();
                emit_alert_text("bench_plugin", "reason", &m).unwrap();
            },
            BatchSize::SmallInput,
        )
    });

    fs::remove_file(tmp).ok();
}

fn tempfile_path() -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "plugin_api_bench_{}.json",
        Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    p
}

criterion_group!(benches, bench_log_event, bench_emit_alert, bench_combined);
criterion_main!(benches);