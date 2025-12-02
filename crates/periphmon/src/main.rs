use anyhow::{Result, Context};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Serialize, Deserialize};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use thiserror::Error;
use overflow_utils::utc_iso8601;
use std::process::Command;

// PeriphConfig and PeripheralEvent structures as previously defined in lib.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeripheralEvent {
    pub timestamp: String,       // observation time (UTC)
    pub vendor_id: String,       // hex string, e.g. "0x1FC9"
    pub product_id: String,      // hex string, e.g. "0x0083"
    pub description: String,     // human readable
    pub flagged: bool,           // true if matched blacklist
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriphConfig {
    pub poll_interval_ms: u64,
    pub max_devices_per_sample: usize,
    pub blacklist: Vec<(String, String)>,
}

impl Default for PeriphConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 2000,
            max_devices_per_sample: 128,
            blacklist: vec![
                ("0x1FC9".into(), "0x0083".into()), // Hak5 Rubber Ducky
                ("0x1209".into(), "0xBAD1".into()), // O.MG Cable
                ("0x1337".into(), "0xDEAD".into()), // Generic BadUSB
            ],
        }
    }
}

// Main API to poll peripherals
pub fn poll_peripherals(cfg: &PeriphConfig) -> Result<Vec<PeripheralEvent>> {
    BACKEND.sample(cfg)
}

// Start the peripheral watcher
pub fn spawn_periph_watcher(cfg: PeriphConfig) -> Receiver<PeripheralEvent> {
    let (tx, rx): (Sender<PeripheralEvent>, Receiver<PeripheralEvent>) = mpsc::channel();
    thread::spawn(move || {
        let interval = Duration::from_millis(cfg.poll_interval_ms.max(100));
        println!("Starting peripheral watcher thread...");

        loop {
            match BACKEND.sample(&cfg) {
                Ok(list) => {
                    for mut ev in list {
                        ev.timestamp = utc_iso8601();
                        println!("Detected Peripheral: {:?}", ev);  // Debugging line
                        let _ = tx.send(ev);
                    }
                }
                Err(_) => {
                    eprintln!("Error sampling peripherals"); // Debugging if error occurs
                }
            }
            thread::sleep(interval);
        }
    });
    rx
}

// Backend trait to define the sample function
trait Backend: Send + Sync {
    fn name(&self) -> &'static str;
    fn sample(&self, cfg: &PeriphConfig) -> Result<Vec<PeripheralEvent>>;
}

// Lazy initialization of the backend based on the operating system
static BACKEND: Lazy<Box<dyn Backend>> = Lazy::new(|| {
    if MacBackend::available() {
        println!("Using macOS backend");
        Box::new(MacBackend)
    } else if LinuxBackend::available() {
        println!("Using Linux backend");
        Box::new(LinuxBackend)
    } else if WindowsBackend::available() {
        println!("Using Windows backend");
        Box::new(WindowsBackend)
    } else {
        println!("No backend available, falling back to NullBackend");
        Box::new(NullBackend)
    }
});

// macOS backend using ioreg (ioreg command for USB peripherals)
struct MacBackend;

impl MacBackend {
    fn available() -> bool { cfg!(target_os = "macos") }

    fn parse_output(out: &str, cfg: &PeriphConfig) -> Vec<PeripheralEvent> {
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"Vendor ID: (\w+).*?Product ID: (\w+).*?Manufacturer: (.*?)Product: (.*?)\n")
                .unwrap()
        });
        let mut events = Vec::new();
        for cap in RE.captures_iter(out) {
            if events.len() >= cfg.max_devices_per_sample { break; }
            let vid = format!("0x{}", &cap[1]);
            let pid = format!("0x{}", &cap[2]);
            let manufacturer = &cap[3];
            let product = &cap[4];

            let flagged = cfg.blacklist.iter().any(|(bv, bp)| *bv == vid && *bp == pid);

            events.push(PeripheralEvent {
                timestamp: utc_iso8601(),
                vendor_id: vid,
                product_id: pid,
                description: format!("{} - {}", manufacturer, product), // Description with manufacturer and product name
                flagged,
            });
        }
        events
    }

    fn sample(&self, cfg: &PeriphConfig) -> Result<Vec<PeripheralEvent>> {
        let out = std::process::Command::new("ioreg")
            .arg("-p")
            .arg("IOUSB")
            .output()
            .context("failed to run ioreg")?;

        let out_str = String::from_utf8_lossy(&out.stdout);

        // Debugging: Print the output from the ioreg command
        println!("ioreg output:\n{}", out_str);

        Ok(Self::parse_output(&out_str, cfg))
    }
}

impl Backend for MacBackend {
    fn name(&self) -> &'static str { "macos_ioreg" }

    fn sample(&self, cfg: &PeriphConfig) -> Result<Vec<PeripheralEvent>> {
        self.sample(cfg)
    }
}

// Linux backend using lsusb
struct LinuxBackend;

impl LinuxBackend {
    fn available() -> bool { cfg!(target_os = "linux") }
}

impl Backend for LinuxBackend {
    fn name(&self) -> &'static str { "linux_lsusb" }
    fn sample(&self, cfg: &PeriphConfig) -> Result<Vec<PeripheralEvent>> {
        let out = std::process::Command::new("lsusb").output()
            .context("failed to run lsusb")?;
        let s = String::from_utf8_lossy(&out.stdout);
        let mut events = Vec::new();
        for line in s.lines() {
            if events.len() >= cfg.max_devices_per_sample { break; }
            if let Some((vid, pid)) = line.split_whitespace()
                .find(|tok| tok.contains(":"))
                .and_then(|tok| {
                    let mut parts = tok.split(":");
                    Some((format!("0x{}", parts.next()?.to_string()),
                          format!("0x{}", parts.next()?.to_string())))
                }) {
                let flagged = cfg.blacklist.iter().any(|(bv, bp)| *bv == vid && *bp == pid);
                events.push(PeripheralEvent {
                    timestamp: utc_iso8601(),
                    vendor_id: vid,
                    product_id: pid,
                    description: "USB peripheral".into(),
                    flagged,
                });
            }
        }
        Ok(events)
    }
}

// Placeholder for Windows backend
struct WindowsBackend;

impl WindowsBackend {
    fn available() -> bool { cfg!(target_os = "windows") }
}

impl Backend for WindowsBackend {
    fn name(&self) -> &'static str { "windows_wmic" }
    fn sample(&self, _cfg: &PeriphConfig) -> Result<Vec<PeripheralEvent>> {
        // Phase I: not implemented
        Ok(vec![])
    }
}

// Null backend as a fallback
struct NullBackend;

impl Backend for NullBackend {
    fn name(&self) -> &'static str { "null" }
    fn sample(&self, _cfg: &PeriphConfig) -> Result<Vec<PeripheralEvent>> {
        Ok(vec![])
    }
}

// Error handling for peripherals
#[derive(Debug, Error)]
pub enum PeriphError {
    #[error("backend unavailable")]
    BackendUnavailable,
}

fn main() -> Result<()> {
    // Initialize the configuration with default values
    let cfg = PeriphConfig::default();

    // Start the peripheral watcher in a separate thread
    let rx = spawn_periph_watcher(cfg);

    // Display events in real-time
    println!("Starting peripheral monitoring...");

    loop {
        match rx.recv() {
            Ok(event) => {
                // Print each event's details
                println!(
                    "[{}] Vendor ID: {}, Product ID: {}, Description: {}, Flagged: {}",
                    event.timestamp, event.vendor_id, event.product_id, event.description, event.flagged
                );
            }
            Err(_) => {
                // If there's an issue receiving events, just continue
                eprintln!("Error receiving event, retrying...");
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}