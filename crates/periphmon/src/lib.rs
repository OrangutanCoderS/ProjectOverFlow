/*!
 Module 18 — Peripheral Monitor (periphmon)
 Goal: Observe attached peripherals (USB, HID) and detect suspicious devices.
 Phase I: observation-only, safe log of vendor/product IDs.
 Security: no blocking, only metadata (no data exfil).
*/

use anyhow::{Result, Context};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Serialize, Deserialize};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use thiserror::Error;

use overflow_utils::utc_iso8601;

/* ============================
   Data model
   ============================ */

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeripheralEvent {
    pub timestamp: String,       // observation time (UTC)
    pub vendor_id: String,       // hex string, e.g. "0x1FC9"
    pub product_id: String,      // hex string, e.g. "0x0083"
    pub description: String,     // human readable
    pub flagged: bool,           // true if matched blacklist
}

impl PeripheralEvent {
    pub fn validate(&self) -> Result<()> {
        if self.vendor_id.is_empty() || self.product_id.is_empty() {
            anyhow::bail!("missing vendor/product ID");
        }
        Ok(())
    }
}

/* ============================
   Config
   ============================ */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriphConfig {
    pub poll_interval_ms: u64,
    pub max_devices_per_sample: usize,
    pub blacklist: Vec<(String, String)>, // (vendor_id, product_id)
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

/* ============================
   Public API
   ============================ */

pub fn poll_peripherals(cfg: &PeriphConfig) -> Result<Vec<PeripheralEvent>> {
    BACKEND.sample(cfg)
}

pub fn spawn_periph_watcher(cfg: PeriphConfig) -> Receiver<PeripheralEvent> {
    let (tx, rx): (Sender<PeripheralEvent>, Receiver<PeripheralEvent>) = mpsc::channel();
    thread::spawn(move || {
        let interval = Duration::from_millis(cfg.poll_interval_ms.max(100));
        loop {
            match BACKEND.sample(&cfg) {
                Ok(list) => {
                    for mut ev in list {
                        ev.timestamp = utc_iso8601();
                        let _ = tx.send(ev);
                    }
                }
                Err(_) => {}
            }
            thread::sleep(interval);
        }
    });
    rx
}

/* ============================
   Backend trait + selection
   ============================ */

trait Backend: Send + Sync {
    fn name(&self) -> &'static str;
    fn sample(&self, cfg: &PeriphConfig) -> Result<Vec<PeripheralEvent>>;
}

static BACKEND: Lazy<Box<dyn Backend>> = Lazy::new(|| {
    if MacBackend::available() {
        Box::new(MacBackend)
    } else if LinuxBackend::available() {
        Box::new(LinuxBackend)
    } else if WindowsBackend::available() {
        Box::new(WindowsBackend)
    } else {
        Box::new(NullBackend)
    }
});

/* ============================
   macOS backend (system_profiler)
   ============================ */

struct MacBackend;

impl MacBackend {
    fn available() -> bool { cfg!(target_os = "macos") }

    fn parse_output(out: &str, cfg: &PeriphConfig) -> Vec<PeripheralEvent> {
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"Vendor ID:\s*0x(?P<vid>[0-9a-fA-F]+).*Product ID:\s*0x(?P<pid>[0-9a-fA-F]+)")
                .unwrap()
        });
        let mut events = Vec::new();
        for cap in RE.captures_iter(out) {
            if events.len() >= cfg.max_devices_per_sample { break; }
            let vid = format!("0x{}", &cap["vid"]);
            let pid = format!("0x{}", &cap["pid"]);
            let flagged = cfg.blacklist.iter().any(|(bv, bp)| *bv == vid && *bp == pid);
            events.push(PeripheralEvent {
                timestamp: utc_iso8601(),
                vendor_id: vid,
                product_id: pid,
                description: "USB peripheral".into(),
                flagged,
            });
        }
        events
    }
}

impl Backend for MacBackend {
    fn name(&self) -> &'static str { "macos_system_profiler" }

    fn sample(&self, cfg: &PeriphConfig) -> Result<Vec<PeripheralEvent>> {
        let out = std::process::Command::new("system_profiler")
            .arg("SPUSBDataType")
            .output()
            .context("failed to run system_profiler")?;
        let s = String::from_utf8_lossy(&out.stdout);
        Ok(Self::parse_output(&s, cfg))
    }
}

/* ============================
   Linux backend (lsusb)
   ============================ */

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

/* ============================
   Windows backend (wmic USB)
   ============================ */

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

/* ============================
   Null backend
   ============================ */

struct NullBackend;
impl Backend for NullBackend {
    fn name(&self) -> &'static str { "null" }
    fn sample(&self, _cfg: &PeriphConfig) -> Result<Vec<PeripheralEvent>> {
        Ok(vec![])
    }
}

/* ============================
   Error surface
   ============================ */
#[derive(Debug, Error)]
pub enum PeriphError {
    #[error("backend unavailable")]
    BackendUnavailable,
}