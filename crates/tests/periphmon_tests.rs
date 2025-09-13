use periphmon::{poll_peripherals, PeriphConfig};

#[test]
fn returns_vector_or_empty() {
    let cfg = PeriphConfig::default();
    let res = poll_peripherals(&cfg);
    assert!(res.is_ok());
}
