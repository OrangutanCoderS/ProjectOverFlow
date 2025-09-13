use daemon::Daemon;
use std::path::PathBuf;

fn main() {
    // Pick config path: env or default file
    let cfg_path = std::env::var("OVERFLOW_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("configs/phase1-observe-only.toml"));

    let d = Daemon::new_from_file(cfg_path).unwrap_or_else(|e| {
        eprintln!("bootstrap: {e}");
        std::process::exit(2)
    });

    // Non-blocking mode for quick smoke runs
    let non_block = std::env::var("OVERFLOW_NON_BLOCKING").ok().is_some();
    if non_block {
        d.run().unwrap_or_else(|e| {
            eprintln!("run: {e}");
            std::process::exit(3)
        });
    } else {
        d.run_until_shutdown().unwrap_or_else(|e| {
            eprintln!("run_until_shutdown: {e}");
            std::process::exit(4)
        });
    }
}
