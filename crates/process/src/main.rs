use process::{ProcessMonitor, ProcMonCfg};
use serde::Serialize;
use std::{time::{Duration, SystemTime, UNIX_EPOCH}, thread};

#[derive(Serialize)]
struct ProcessInfoOut<'a> {
    pid: i32,
    cpu_pct: f32,
    mem_mb: f32,
    user: &'a str,
    cmd: &'a str,
}

#[derive(Serialize)]
struct ProcessSnapshotOut<'a> {
    ts_ms: u64,
    processes: Vec<ProcessInfoOut<'a>>,
}

#[derive(Serialize)]
struct Envelope<'a> {
    event_type: &'a str,
    data: ProcessSnapshotOut<'a>,
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn main() {
    let mut mon = ProcessMonitor::new(ProcMonCfg::default())
        .expect("Failed to init ProcessMonitor");

    loop {
        // Capture snapshot
        let snap = mon.capture().expect("Failed to capture snapshot");

        // Collect processes
        let mut out: Vec<ProcessInfoOut> = snap.map
            .values()
            .map(|info| ProcessInfoOut {
                pid: info.base.pid,
                cpu_pct: info.cpu_pct,
                mem_mb: info.mem_mb,
                user: &info.user,
                cmd: &info.name,
            })
            .collect();

        // Sort by CPU
        out.sort_by(|a, b| b.cpu_pct.total_cmp(&a.cpu_pct));

        // Build snapshot envelope
        let env = Envelope {
            event_type: "process_snapshot",
            data: ProcessSnapshotOut {
                ts_ms: timestamp_ms(),
                processes: out,
            },
        };

        // Print as JSON line (JSONL)
        println!("{}", serde_json::to_string(&env).unwrap());

        thread::sleep(Duration::from_millis(500));
    }
}