use anyhow::Result;
use clipboard::{ClipboardConfig, spawn_clipboard_watcher};

fn main() -> Result<()> {
    // Local config for the demo binary
    let mut cfg = ClipboardConfig::default();
    cfg.log_plaintext = true; // show actual clipboard content

    // Spawn background watcher thread
    let rx = spawn_clipboard_watcher(cfg.clone());

    println!("Starting clipboard monitoring...\n");

    loop {
        match rx.recv() {
            Ok(event) => {
                if cfg.log_plaintext {
                    println!(
                        "[{}] event={}, len={}, entropy={:.3}, type={}, source={}, content={}",
                        event.timestamp,
                        event.event,
                        event.length,
                        event.entropy_score,
                        event.data_type,
                        event.source,
                        event.content,
                    );
                } else {
                    println!(
                        "[{}] event={}, hash={}, len={}, entropy={:.3}, type={}, source={}",
                        event.timestamp,
                        event.event,
                        event.content_hash,
                        event.length,
                        event.entropy_score,
                        event.data_type,
                        event.source,
                    );
                }
            }
            Err(err) => {
                eprintln!("clipboard watcher channel closed: {err}");
                break;
            }
        }
    }

    Ok(())
}