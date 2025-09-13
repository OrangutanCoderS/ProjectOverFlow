use criterion::{black_box, criterion_group, criterion_main, Criterion};
use netmon::NetworkConnectionInfo;

const LSOF_CORPUS: &str = r#"COMMAND     PID USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
Google      8123 alice  123u IPv4 0x1234567890      0t0   TCP 127.0.0.1:54892->104.26.9.2:443 (ESTABLISHED)
node        7777 bob    33u  IPv6  0xabcd            0t0   TCP *:7000 (LISTEN)
mDNSRespon  111  _mdns  22u  IPv4 0xaabbcc          0t0   UDP 224.0.0.251:5353
# repeat lines to simulate a large output...
"#;

fn bench_validate(c: &mut Criterion) {
    c.bench_function("netmon validate struct", |b| {
        b.iter(|| {
            let info = NetworkConnectionInfo {
                timestamp: "2025-01-01T00:00:00Z".into(),
                pid: 8123,
                process: "Google".into(),
                user: "alice".into(),
                local_ip: "127.0.0.1".into(),
                local_port: 54892,
                remote_ip: "104.26.9.2".into(),
                remote_port: 443,
                protocol: "TCP".into(),
                state: "ESTABLISHED".into(),
                bytes_sent: None,
                bytes_received: None,
                interface: None,
                dns_query: None,
            };
            black_box(info.validate().unwrap());
        })
    });
}

criterion_group!(benches, bench_validate);
criterion_main!(benches);