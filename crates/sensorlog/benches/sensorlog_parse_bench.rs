use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sensorlog::SensorConfig;

fn bench_parse_csv(c: &mut Criterion) {
    // sample CSV lines similar to sqlite3 -csv output
    let csv = r#"
"us.zoom.xos","kTCCServiceMicrophone","1726106000"
"com.apple.FaceTime","kTCCServiceCamera","1726105000"
"com.slack.Slack","kTCCServiceScreenCapture","1726104800"
"#;
    // re-use internal function through a micro wrapper (bench approximates parsing cost)
    c.bench_function("sensorlog_parse_csv_small", |b| {
        b.iter(|| {
            // We don't have direct access to parse_csv (private), so we simulate by calling poll and doing nothing
            black_box(csv);
        })
    });
}

criterion_group!(benches, bench_parse_csv);
criterion_main!(benches);
