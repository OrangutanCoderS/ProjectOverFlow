use criterion::{criterion_group, criterion_main, Criterion, black_box};
use system::thermal_logger::{parse_powermetrics_thermal, ThermalCfg, ThermalLogger};

fn bench_parser(c: &mut Criterion) {
    let sample = r#"
CPU die temperature: 37.6 C
GPU die temperature: 38.1 C
SoC die temperature: 36.0 C
CPU Thermal level: Warning
"#;
    c.bench_function("thermal_parser_powermetrics", |b| {
        b.iter(|| {
            let r = parse_powermetrics_thermal(black_box(sample));
            black_box(r);
        });
    });
}

fn bench_capture(c: &mut Criterion) {
    let logger = ThermalLogger::new(ThermalCfg::default());
    if logger.is_err() {
        // Skip on non-macOS to avoid skewing CI
        return;
    }
    let logger = logger.unwrap();

    c.bench_function("thermal_capture_powermetrics", |b| {
        b.iter(|| {
            let _ = logger.capture();
        });
    });
}

criterion_group!(benches, bench_parser, bench_capture);
criterion_main!(benches);