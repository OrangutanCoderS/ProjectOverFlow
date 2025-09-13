use system::battery_monitor::{BatteryCfg, BatteryMonitor, parse_pmset, parse_ioreg};
use overflow_core::{AnyEvent, Event};

#[test]
fn pmset_parser_happy_paths() {
    let sample = r#"
Now drawing from 'AC Power'
 -InternalBattery-0 (id=123456)    83%; charging; 0:58 remaining present: true
"#;
    let f = parse_pmset(sample);
    assert_eq!(f.percentage, Some(83.0));
    assert_eq!(f.charging, Some(true));
    assert_eq!(f.time_remaining_min, Some(58));

    let sample2 = r#"
Now drawing from 'Battery Power'
 -InternalBattery-0 (id=123456)    53%; discharging; (no estimate) present: true
"#;
    let f2 = parse_pmset(sample2);
    assert_eq!(f2.percentage, Some(53.0));
    assert_eq!(f2.charging, Some(false));
    assert_eq!(f2.time_remaining_min, None);
}

#[test]
fn ioreg_parser_basic() {
    let sample = r#"
    CycleCount = 532
    Temperature = 2980
    Voltage = 12123
    PermanentFailureStatus = 0
"#;
    let f = parse_ioreg(sample);
    assert_eq!(f.cycle_count, Some(532));
    assert!(f.temperature_c.unwrap() >= 0.0); // sanity bound applied
    assert_eq!(f.voltage_mv, Some(12123));
    assert_eq!(f.health.as_deref(), Some("Normal"));
}

#[test]
fn tracker_constructs_and_emits_or_soft_errors() {
    let mon = BatteryMonitor::new(BatteryCfg::default());
    if mon.is_err() {
        // non-macOS: expected
        return;
    }
    let mon = mon.unwrap();

    let res = mon.snapshot();
    match res {
        Ok(AnyEvent::Battery(e)) => {
            assert!(e.validate().is_ok());
            assert!(
                e.percentage >= 0.0 && e.percentage <= 100.0,
                "percentage out of bounds: {}",
                e.percentage
            );
        }
        Ok(_) => panic!("unexpected event type"),
        Err(_) => {
            // Acceptable in CI/sandbox if subprocess access is blocked.
        }
    }
}