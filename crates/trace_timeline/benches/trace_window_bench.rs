use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::prelude::*;
use trace_timeline::{
    EventKind, TimelineEvent, TimelineSource, TraceTimelineConfig, TraceTimelineStore,
};

fn build_store(num_events: usize) -> TraceTimelineStore {
    let cfg = TraceTimelineConfig::default().with_max_events(num_events + 10);
    let mut store = TraceTimelineStore::new(cfg);

    // Simple monotonically increasing timestamps with some jitter.
    let mut ts = 0_i64;
    let mut rng = StdRng::seed_from_u64(42);

    for _ in 0..num_events {
        let jitter: i64 = rng.gen_range(0..1_000);
        ts += 1_000_000 + jitter; // ~1ms apart with jitter

        let event = TimelineEvent::new(ts, EventKind::PluginInvocation, TimelineSource::PluginEvents);
        store.insert(event).unwrap();
    }

    store
}

fn bench_timeline_windows(c: &mut Criterion) {
    let store = build_store(100_000);
    let mid_ts = store.events()[store.len() / 2].ts_nanos;

    c.bench_function("trace_timeline_window_by_range_1k", |b| {
        b.iter(|| {
            let start = mid_ts - 500_000_000;
            let end = mid_ts + 500_000_000;
            let window = store.window_by_range(start, end);
            black_box(window.len());
        })
    });

    c.bench_function("trace_timeline_last_n_1k", |b| {
        b.iter(|| {
            let window = store.last_n(1_000);
            black_box(window.len());
        })
    });
}

criterion_group!(benches, bench_timeline_windows);
criterion_main!(benches);