use system::thermal_logger::{parse_powermetrics_thermal, ThermalCfg, ThermalLogger};

#[test]
fn parse_powermetrics_basic() {
    let sample = r#"
==== Sample ====
CPU die temperature: 37.6 C
GPU die temperature: 38.1 C
SoC die temperature: 36.0 C
CPU Thermal level: Warning
"#;
    let s = parse_powermetrics_thermal(sample);
    assert_eq!(s.cpu_temp_c, Some(37.6));
    assert_eq!(s.gpu_temp_c, Some(38.1));
    assert_eq!(s.soc_temp_c, Some(36.0));
    assert_eq!(s.throttling, Some(true));
}

#[test]
fn parse_powermetrics_numeric_level() {
    let sample = r#"
CPU die temperature: 45.0 C
CPU Thermal level: 0
"#;
    let s = parse_powermetrics_thermal(sample);
    assert_eq!(s.cpu_temp_c, Some(45.0));
    assert_eq!(s.throttling, Some(false));
}

#[test]
fn logger_constructs_and_runs_or_soft_errors() {
    // Works on macOS; on other OS return Unsupported (we exit the test).
    let logger = ThermalLogger::new(ThermalCfg::default());
    if logger.is_err() { return; }
    let logger = logger.unwrap();

    // We don't assert actual values (device/permissions dependent),
    // but ensure we either get a sane snapshot or a controlled error.
    let res = logger.capture();
    match res {
        Ok(snap) => {
            for v in [snap.cpu_temp_c, snap.gpu_temp_c, snap.soc_temp_c].into_iter().flatten() {
                assert!((0.0..=120.0).contains(&v));
            }
        }
        Err(_) => {
            // Acceptable on CI/sandbox if powermetrics access is blocked.
        }
    }
}