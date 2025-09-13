//! Module 10 — Battery Monitor (Phase I)
//! macOS-first snapshot using `pmset` (rootless) + optional `ioreg` enrichment.
//! Emits `AnyEvent::Battery(BatteryEvent)`.
//!
//! On non-macOS: returns `Unsupported`.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use regex::Regex;
use thiserror::Error;
use tracing::{debug, warn};

use overflow_core::{AnyEvent, BaseEvent, BatteryEvent, Event};

#[derive(Debug, Error)]
pub enum BatteryError {
    #[error("unsupported platform")]
    Unsupported,
    #[error("subprocess: {0}")]
    Subprocess(String),
    #[error("parse: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, BatteryError>;

/// Public configuration
#[derive(Debug, Clone, Copy)]
pub struct BatteryCfg {
    /// Timeout for pmset / ioreg subprocess calls.
    pub timeout_ms: u64,
    /// If true, try to enrich with ioreg extra fields (health/cycles/voltage/temp).
    pub enrich_ioreg: bool,
}

impl Default for BatteryCfg {
    fn default() -> Self {
        Self {
            timeout_ms: 500,
            enrich_ioreg: true,
        }
    }
}

pub struct BatteryMonitor {
    cfg: BatteryCfg,
}

impl BatteryMonitor {
    pub fn new(cfg: BatteryCfg) -> Result<Self> {
        #[cfg(target_os = "macos")]
        {
            Ok(Self { cfg })
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err(BatteryError::Unsupported)
        }
    }

    /// Capture one battery snapshot and return as AnyEvent.
    pub fn snapshot(&self) -> Result<AnyEvent> {
        #[cfg(not(target_os = "macos"))]
        {
            return Err(BatteryError::Unsupported);
        }

        #[cfg(target_os = "macos")]
        {
            let t0 = Instant::now();
            let timeout = Duration::from_millis(self.cfg.timeout_ms);

            let base = BaseEvent::new(0, "battery_monitor");

            // pmset is the primary source for percentage, charging, time remaining
            let pm = self.sample_pmset(timeout)?;

            // ioreg enrichment (best-effort)
            let enr = if self.cfg.enrich_ioreg {
                self.sample_ioreg(timeout).ok()
            } else {
                None
            };

            let evt = BatteryEvent {
                base,
                percentage: pm.percentage.unwrap_or(0.0).clamp(0.0, 100.0),
                charging: pm.charging.unwrap_or(false),
                cycle_count: enr.as_ref().and_then(|e| e.cycle_count),
                temperature_c: enr.as_ref().and_then(|e| e.temperature_c),
                health: enr.as_ref().and_then(|e| e.health.clone()),
                voltage_mv: enr.as_ref().and_then(|e| e.voltage_mv),
                time_remaining_min: pm.time_remaining_min,
            };

            if let Err(e) = evt.validate() {
                return Err(BatteryError::Parse(e.to_string()));
            }

            let elapsed = t0.elapsed().as_millis() as u64;
            if elapsed > self.cfg.timeout_ms {
                warn!(elapsed_ms = elapsed, "battery snapshot exceeded soft budget");
            } else {
                debug!(elapsed_ms = elapsed, "battery snapshot ok");
            }

            Ok(AnyEvent::Battery(evt))
        }
    }

    // ---------- subprocess helpers ----------

    #[cfg(target_os = "macos")]
    fn run_with_timeout(
        &self,
        program: &str,
        args: &[&str],
        timeout: Duration,
    ) -> Result<(i32, Vec<u8>, Vec<u8>)> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| BatteryError::Subprocess(format!("{program} spawn: {e}")))?;

        let start = Instant::now();
        loop {
            if let Some(status) =
                child
                    .try_wait()
                    .map_err(|e| BatteryError::Subprocess(format!("try_wait: {e}")))? {
                let mut out = Vec::new();
                let mut err = Vec::new();
                if let Some(mut s) = child.stdout.take() {
                    let _ = s.read_to_end(&mut out);
                }
                if let Some(mut s) = child.stderr.take() {
                    let _ = s.read_to_end(&mut err);
                }
                return Ok((status.code().unwrap_or(-1), out, err));
            }
            if start.elapsed() >= timeout {
                let _ = child.kill();
                let _ = child.wait();
                let mut out = Vec::new();
                let mut err = Vec::new();
                if let Some(mut s) = child.stdout.take() {
                    let _ = s.read_to_end(&mut out);
                }
                if let Some(mut s) = child.stderr.take() {
                    let _ = s.read_to_end(&mut err);
                }
                return Err(BatteryError::Subprocess(format!(
                    "{program} timed out after {:?}. stderr: {}",
                    timeout,
                    String::from_utf8_lossy(&err)
                )));
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    // ---------- pmset parser ----------

    #[cfg(target_os = "macos")]
    fn sample_pmset(&self, timeout: Duration) -> Result<PmsetFields> {
        // Example `pmset -g batt` output:
        //
        // Now drawing from 'AC Power'
        //  -InternalBattery-0 (id=xxxx)    83%; charging; 0:58 remaining present: true
        //
        // or on battery:
        // Now drawing from 'Battery Power'
        //  -InternalBattery-0 (id=xxxx)    53%; discharging; (no estimate) present: true
        let (code, out, err) =
            self.run_with_timeout("/usr/bin/pmset", &["-g", "batt"], timeout)?;
        if code != 0 {
            return Err(BatteryError::Subprocess(format!(
                "pmset exit={code}, stderr={}",
                String::from_utf8_lossy(&err)
            )));
        }
        let s = String::from_utf8_lossy(&out);
        Ok(parse_pmset(&s))
    }

    // ---------- ioreg enrichment ----------

    #[cfg(target_os = "macos")]
    fn sample_ioreg(&self, timeout: Duration) -> Result<IoregFields> {
        // `ioreg -rc AppleSmartBattery` exposes CycleCount, Temperature (in 0.1°C), Voltage (mV), etc.
        let (code, out, err) = self.run_with_timeout(
            "/usr/sbin/ioreg",
            &["-rc", "AppleSmartBattery"],
            timeout,
        )?;
        if code != 0 {
            return Err(BatteryError::Subprocess(format!(
                "ioreg exit={code}, stderr={}",
                String::from_utf8_lossy(&err)
            )));
        }
        let s = String::from_utf8_lossy(&out);
        Ok(parse_ioreg(&s))
    }
}

// ---------- Parsed fields (public for tests) ----------

#[derive(Debug, Default)]
pub struct PmsetFields {
    pub percentage: Option<f32>,
    pub charging: Option<bool>,
    pub time_remaining_min: Option<u32>,
}

#[derive(Debug, Default)]
pub struct IoregFields {
    pub cycle_count: Option<u32>,
    pub temperature_c: Option<f32>,
    pub health: Option<String>,
    pub voltage_mv: Option<u32>,
}

// ---------- Parsers (unit-tested) ----------

/// Parse `pmset -g batt` best-effort.
pub fn parse_pmset(s: &str) -> PmsetFields {
    let mut f = PmsetFields::default();

    // percentage (e.g. "83%; charging;" or "53%; discharging;")
    if let Some(cap) = Regex::new(r"(?m)(\d{1,3})%;").unwrap().captures(s) {
        if let Ok(p) = cap.get(1).unwrap().as_str().parse::<u32>() {
            f.percentage = Some((p as f32).clamp(0.0, 100.0));
        }
    }

    // charging / discharging
    if s.contains("charging;") || s.contains("AC Power'") {
        f.charging = Some(true);
    } else if s.contains("discharging;") || s.contains("Battery Power'") {
        f.charging = Some(false);
    }

    // time remaining like "0:58 remaining" or "1:42 remaining"
    if let Some(cap) = Regex::new(r"(\d+):(\d+)\s+remaining").unwrap().captures(s) {
        let h: u32 = cap.get(1).unwrap().as_str().parse().unwrap_or(0);
        let m: u32 = cap.get(2).unwrap().as_str().parse().unwrap_or(0);
        f.time_remaining_min = Some(h.saturating_mul(60).saturating_add(m));
    } else if s.contains("(no estimate)") {
        f.time_remaining_min = None; // unknown
    }

    f
}

/// Parse `ioreg -rc AppleSmartBattery` best-effort.
pub fn parse_ioreg(s: &str) -> IoregFields {
    let mut f = IoregFields::default();

    // CycleCount = 532
    if let Some(cap) = Regex::new(r"CycleCount\s*=\s*(\d+)").unwrap().captures(s) {
        f.cycle_count = cap
            .get(1)
            .and_then(|m| m.as_str().parse::<u32>().ok());
    }

    // Temperature = 2980  (heuristic: 0.1 K units -> °C ≈ raw/10 - 273.15)
    if let Some(cap) = Regex::new(r"Temperature\s*=\s*(\d+)").unwrap().captures(s) {
        if let Ok(raw) = cap.get(1).unwrap().as_str().parse::<u32>() {
            let t_c = (raw as f32) / 10.0 - 273.15;
            let t_c = t_c.clamp(0.0, 100.0); // sanity clamp
            f.temperature_c = Some(t_c);
        }
    }

    // Voltage = 12123 (mV)
    if let Some(cap) = Regex::new(r"Voltage\s*=\s*(\d+)").unwrap().captures(s) {
        f.voltage_mv = cap
            .get(1)
            .and_then(|m| m.as_str().parse::<u32>().ok());
    }

    // PermanentFailureStatus -> simple health mapping
    if let Some(cap) = Regex::new(r"PermanentFailureStatus\s*=\s*(\d+)").unwrap().captures(s) {
        if cap
            .get(1)
            .and_then(|m| m.as_str().parse::<u32>().ok())
            .unwrap_or(1)
            == 0
        {
            f.health = Some("Normal".to_string());
        } else {
            f.health = Some("Service Recommended".to_string());
        }
    }

    f
}