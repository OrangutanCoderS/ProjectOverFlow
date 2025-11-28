use autonomous_runtime::{
    AutonomousRuntime,
    NullActionSink,
    NullEventSource,
    RuntimeCommand,
    RuntimeConfig,
};
use autonomous_runtime::clock::NoopClock;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_empty_tick(c: &mut Criterion) {
    // -------------------------------------------------------------
    // VALID CONFIG — MUST PASS VALIDATION
    // -------------------------------------------------------------
    let mut cfg = RuntimeConfig {
        tick_interval_ms: 1,            // must be > 0 (validator requirement)
        max_events_per_tick: 256,
        flush_interval_ticks: 1000,
        fail_fast: true,
    };

    // Ensure config is valid BEFORE runtime starts
    cfg.validate().expect("RuntimeConfig validation failed");

    // -------------------------------------------------------------
    // NULL SOURCE / SINK / CLOCK (NOOP) — FASTEST POSSIBLE LOOP
    // -------------------------------------------------------------
    let source = NullEventSource;
    let sink = NullActionSink;
    let clock = NoopClock::default();

    // -------------------------------------------------------------
    // CREATE RUNTIME
    // -------------------------------------------------------------
    let mut rt = AutonomousRuntime::new(cfg, source, sink, clock)
        .expect("Runtime initialization failed");

    // -------------------------------------------------------------
    // START RUNTIME (HYBRID MODE SAFE)
    // -------------------------------------------------------------
    rt.apply_command(RuntimeCommand::Start)
        .expect("Runtime start() should succeed");

    // -------------------------------------------------------------
    // BENCHMARK LOOP
    // -------------------------------------------------------------
    c.bench_function("autonomous_runtime_empty_tick", |b| {
        b.iter(|| {
            let stats = rt.tick_once().expect("tick_once() should succeed");
            black_box(stats);
        });
    });
}

criterion_group!(benches, bench_empty_tick);
criterion_main!(benches);