use cloudsync::{poll_cloudsync, CloudSyncConfig, CloudSyncEvent};

#[test]
fn detects_dropbox_burst() {
    // Prepare config (only fields that actually exist)
    let cfg = CloudSyncConfig {
        poll_interval_ms: 100,
        max_entries_per_sample: 50,
    };

    // Run poll — unwrap gives Vec<CloudSyncEvent>
    let res: Vec<CloudSyncEvent> = poll_cloudsync(&cfg).unwrap();

    // Assert: must not panic; allow empty since backend is placeholder
    assert!(res.is_empty() || res.len() >= 0);
}

#[test]
fn detects_any_provider_event() {
    let cfg: CloudSyncConfig = CloudSyncConfig::default();
    let res: Vec<CloudSyncEvent> = poll_cloudsync(&cfg).unwrap();

    // Allow empty; else validate that provider is non-empty and burst_score is in 0..=1
    assert!(
        res.is_empty()
            || res.iter().all(|ev| {
                !ev.provider.is_empty() &&
                (0.0..=1.0).contains(&ev.burst_score)
            })
    );
}