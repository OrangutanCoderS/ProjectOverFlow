use std::env;
use std::path::PathBuf;
use std::time::Duration;

use filemon::{FileMonCfg, FileMonitor};
use overflow_core::{AnyEvent, FileAccessEvent, FileOp};

fn main() {
    let args: Vec<String> = env::args().collect();
    let root = args.get(1)
        .map(|s| PathBuf::from(s))
        .unwrap_or(dirs::home_dir().unwrap_or(PathBuf::from("/tmp")));

    println!("=== ShadowTrace File Monitor (Polling Only) ===");
    println!("Watching root: {:?}", root);

    // Force polling
    let cfg = FileMonCfg {
        roots: vec![root],
        run_window_ms: 2000,    // 2s window
        poll_interval_ms: 500,  // check twice per second
        enable_poll_fallback: true,
        ..Default::default()
    };

    let mon = FileMonitor::new(cfg).unwrap();

    loop {
        match mon.snapshot() {
            Ok(events) if !events.is_empty() => {
                for e in events {
                    if let AnyEvent::FileAccess(f) = e {
                        print_event(&f);
                    }
                }
            }
            Ok(_) => {
                println!("(No events this window — polling backend)");
            }
            Err(e) => {
                eprintln!("File monitor error: {e}");
                break;
            }
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

fn print_event(f: &FileAccessEvent) {
    let op = match f.op {
        FileOp::Open => "OPEN",
        FileOp::Write => "WRITE",
        FileOp::Delete => "DELETE",
        FileOp::Rename => "RENAME",
        FileOp::Exec => "EXEC",
        FileOp::Read => "READ",
        _ => "OTHER",
    };
    println!("[pid={}] {} {}", f.base.pid, op, f.path);
}