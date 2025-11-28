use netmon::{NetworkConfig, spawn_polling};
use std::{env, time::Duration};

fn format_connection(conn: &netmon::NetworkConnectionInfo) -> String {
    format!(
        "{:<6} {:<15} {:<20} {:<5} {:<22} -> {:<22} {:<12} {:>8}/{:<8} {:<5}",
        conn.pid,
        conn.user,
        conn.process,
        conn.protocol,
        format!("{}:{}", conn.local_ip, conn.local_port),
        format!("{}:{}", conn.remote_ip, conn.remote_port),
        conn.state.to_uppercase(),
        conn.bytes_received.unwrap_or(0),
        conn.bytes_sent.unwrap_or(0),
        conn.interface.clone().unwrap_or_else(|| "-".to_string())
    )
}

fn main() {
    // interval (ms) can be passed as arg, else default = 1000
    let args: Vec<String> = env::args().collect();
    let interval = if args.len() > 1 {
        args[1].parse::<u64>().unwrap_or(1000)
    } else {
        1000
    };

    let cfg = NetworkConfig {
        interval_ms: interval,
        ..Default::default()
    };

    println!("=== ShadowTrace Network Monitor ===");
    println!("Backend in use: {}", netmon::backend_name());

    // Print header once
    println!(
        "{:<6} {:<15} {:<20} {:<5} {:<22} -> {:<22} {:<12} {:>8}/{:<8} {:<5}",
        "PID", "USER", "PROCESS", "PROTO", "LOCAL", "REMOTE", "STATE", "IN", "OUT", "IFACE"
    );
    println!("{}", "-".repeat(120));

    let rx = spawn_polling(cfg);

    loop {
        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(conn) => {
                println!("{}", format_connection(&conn));
            }
            Err(_) => {
                println!("(No connections seen in last 5s)");
            }
        }
    }
}