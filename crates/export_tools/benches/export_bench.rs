use criterion::{criterion_group, criterion_main, Criterion, black_box};
use export_tools::formats::{csv, json};
use chrono::Utc;
use serde::Serialize;
use tempfile::tempdir;

#[derive(Serialize, Clone)]
struct Row {
    id: u32,
    user: String,
    note: String,
    when: chrono::DateTime<Utc>,
}

fn make_rows(n: usize) -> Vec<Row> {
    (0..n).map(|i| Row {
        id: i as u32,
        user: format!("user_{i}"),
        note: "lorem ipsum dolor sit amet".into(),
        when: Utc::now(),
    }).collect()
}

fn bench_exports(c: &mut Criterion) {
    let rows = make_rows(5_000);
    let dir = tempdir().unwrap();

    c.bench_function("json_pretty_5k", |b| {
        let cfg = json::JsonConfig { pretty: true, redact_keys: vec![] };
        b.iter(|| {
            let p = dir.path().join("out.json");
            json::export_json(black_box(&p), black_box(&rows), black_box(&cfg)).unwrap();
        });
    });

    c.bench_function("json_compact_5k", |b| {
        let cfg = json::JsonConfig { pretty: false, redact_keys: vec![] };
        b.iter(|| {
            let p = dir.path().join("out_compact.json");
            json::export_json(black_box(&p), black_box(&rows), black_box(&cfg)).unwrap();
        });
    });

    c.bench_function("csv_5k", |b| {
        let cfg = csv::CsvConfig { header: true, redact_keys: vec![] };
        b.iter(|| {
            let p = dir.path().join("out.csv");
            csv::export_csv(black_box(&p), black_box(&rows), black_box(&cfg)).unwrap();
        });
    });
}

criterion_group!(benches, bench_exports);
criterion_main!(benches);
