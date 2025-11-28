use std::io::Write;

use criterion::{criterion_group, criterion_main, Criterion};
use policy_diff_viewer::{diff_files, PolicyDiffFormat};
use tempfile::NamedTempFile;

fn bench_json_policy_diff(c: &mut Criterion) {
    let mut f_current = NamedTempFile::new().expect("tempfile");
    let mut f_proposed = NamedTempFile::new().expect("tempfile");

    // Slightly bigger structure to make the diff non-trivial.
    write!(
        f_current,
        r#"{{
            "rules": {{
                "high_entropy_flag": {{
                    "chain": ["plugin_A", "plugin_B"],
                    "timeout_ms": 1500
                }},
                "usb_spike": {{
                    "chain": ["plugin_C", "plugin_D"],
                    "timeout_ms": 1000
                }}
            }}
        }}"#
    )
    .unwrap();

    write!(
        f_proposed,
        r#"{{
            "rules": {{
                "high_entropy_flag": {{
                    "chain": ["plugin_A", "plugin_X", "plugin_B"],
                    "timeout_ms": 1200
                }},
                "usb_spike": {{
                    "chain": ["plugin_C", "plugin_D"],
                    "timeout_ms": 1000
                }}
            }}
        }}"#
    )
    .unwrap();

    let p_current = f_current.into_temp_path();
    let p_proposed = f_proposed.into_temp_path();

    c.bench_function("policy_json_diff", |b| {
        b.iter(|| {
            let _ = diff_files(&p_current, &p_proposed, Some(PolicyDiffFormat::Json)).unwrap();
        });
    });
}

criterion_group!(benches, bench_json_policy_diff);
criterion_main!(benches);