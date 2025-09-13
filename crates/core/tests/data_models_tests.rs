use overflow_core::*;
use serde_json::Value;

#[test]
fn process_info_ok_and_json_roundtrip() {
    let p = ProcessInfo {
        base: BaseEvent::new(1234, "process"),
        name: "Safari".into(),
        ppid: 1,
        user: "root".into(),
        cpu_pct: 10.5,
        mem_mb: 256.0,
        threads: 5,
        cmdline: vec!["/Applications/Safari.app/Contents/MacOS/Safari".into()],
        start_time_ms: 1_700_000_000_000,
    };
    p.validate().unwrap();
    let s = p.to_json().unwrap();
    let v: Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v["name"], "Safari");
}

#[test]
fn file_access_entropy_bounds_enforced() {
    let mut f = FileAccessEvent {
        base: BaseEvent::new(999, "filemon"),
        path: "/etc/hosts".into(),
        op: FileOp::Read,
        entropy: None,
        plugin_context: None,
    };
    f.validate().unwrap();
    f.entropy = Some(11.0);
    assert!(f.validate().is_err());
}

#[test]
fn network_event_roundtrip() {
    let e = AnyEvent::Network(NetworkEvent {
        base: BaseEvent::new(555, "netmon"),
        proto: NetProto::Tcp,
        remote_addr: "1.2.3.4:443".into(),
        local_addr: Some("10.0.0.8:52555".into()),
        dns_name: Some("example.com".into()),
    });
    e.validate().unwrap();
    let s = e.to_json().unwrap();
    let de: AnyEvent = serde_json::from_str(&s).unwrap();
    assert_eq!(de, e);
}

#[test]
fn plugin_trigger_flags_default_and_traits() {
    let t = PluginTrigger {
        base: BaseEvent::new(0, "plugin"),
        plugin: "anomaly-detector".into(),
        reason: "threshold".into(),
        score: 7.2,
        ..PluginTrigger {
            base: BaseEvent::new(0, "plugin"),
            plugin: String::new(),
            reason: String::new(),
            score: 0.0,
            flags: TriggerFlags::default(),
            context_summary: None,
        }
    };
    t.validate().unwrap();
    // Debug/Clone/PartialEq/Eq exist:
    let _dbg = format!("{:?}", t.flags);
    let _clone = t.flags.clone();
    assert_eq!(t.flags, _clone);
}

#[test]
fn secure_mode_state_eq_is_not_derived_but_partial_eq_is() {
    let a = SecureModeState {
        base: BaseEvent::new(0, "sms"),
        active: true,
        triggered_by: Some("plugin-x".into()),
        trigger_score: Some(5.0),
    };
    let b = a.clone();
    assert_eq!(a, b); // PartialEq works
}
