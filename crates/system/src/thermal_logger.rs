//! Module 11 — Thermal Logger (Phase I, Option 2)
//!
//! Goal: Collect CPU/GPU/SOC temperatures on macOS using `powermetrics` with a
//! resilient parser and a sane timeout. Linux/Windows currently return
//! `Unsupported`.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use regex::Regex;
use thiserror::Error;
use tracing::{debug, warn};

/// Local result type for this module.
pub type Result<T> = std::result::Result<T, ThermalError>;

#[derive(Debug, Error)]
pub enum ThermalError {
    #[error("unsupported platform")]
    Unsupported,
    #[error("subprocess: {0}")]
    Subprocess(String),
    #[error("parse: {0}")]
    Parse(String),
}

/// Public configuration for the logger.
#[derive(Debug, Clone, Copy)]
pub struct ThermalCfg {
    /// Total time budget for one capture (ms).
    pub timeout_ms: u64,
    /// Prefer `powermetrics --samplers thermal` if present, else `--samplers smc`.
    pub prefer_thermal_sampler: bool,
}

impl Default for ThermalCfg {
    fn default() -> Self {
        Self {
            timeout_ms: 700,
            prefer_thermal_sampler: true,
        }
    }
}

/// Local snapshot struct.
#[derive(Debug, Clone, Default)]
pub struct ThermalSnapshot {
    /// CPU die temperature in °C (if available).
    pub cpu_temp_c: Option<f32>,
    /// GPU temperature in °C (if available).
    pub gpu_temp_c: Option<f32>,
    /// SoC/Package temperature in °C (if available).
    pub soc_temp_c: Option<f32>,
    /// Whether we observed a throttle indication (best-effort).
    pub throttling: Option<bool>,
}

impl ThermalSnapshot {
    /// Helper to check if we captured any useful data.
    pub fn is_meaningful(&self) -> bool {
        self.cpu_temp_c.is_some() ||
        self.gpu_temp_c.is_some() ||
        self.soc_temp_c.is_some()
    }
}

pub struct ThermalLogger {
    cfg: ThermalCfg,
}

impl ThermalLogger {
    /// Construct a logger (macOS only for now).
    pub fn new(cfg: ThermalCfg) -> Result<Self> {
        #[cfg(target_os = "macos")]
        {
            Ok(Self { cfg })
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err(ThermalError::Unsupported)
        }
    }

    /// Capture a single snapshot.
    pub fn capture(&self) -> Result<ThermalSnapshot> {
        #[cfg(not(target_os = "macos"))]
        {
            return Err(ThermalError::Unsupported);
        }

        #[cfg(target_os = "macos")]
        {
            let t0 = Instant::now();
            let timeout = Duration::from_millis(self.cfg.timeout_ms);

            // Prefer `thermal` sampler (Apple Silicon); fallback to `smc`.
            let text = if self.cfg.prefer_thermal_sampler {
                self.run_powermetrics(&["-n", "1", "-i", "100", "--samplers", "thermal"], timeout)
                    .or_else(|e| {
                        debug!(err = %e, "thermal sampler failed; trying smc");
                        self.run_powermetrics(&["-n", "1", "-i", "100", "--samplers", "smc"], timeout)
                    })?
            } else {
                match self.run_powermetrics(&["-n", "1", "-i", "100", "--samplers", "smc"], timeout) {
                    Ok(s) => s,
                    Err(e1) => {
                        debug!(err = %e1, "smc sampler failed; trying thermal");
                        self.run_powermetrics(&["-n", "1", "-i", "100", "--samplers", "thermal"], timeout)?
                    }
                }
            };

            let snap = parse_powermetrics_thermal(&text);

            let elapsed = t0.elapsed().as_millis() as u64;
            if elapsed > self.cfg.timeout_ms {
                warn!(elapsed_ms = elapsed, "thermal capture exceeded soft budget");
            } else {
                debug!(elapsed_ms = elapsed, "thermal capture ok");
            }

            Ok(snap)
        }
    }

    #[cfg(target_os = "macos")]
    fn run_powermetrics(&self, args: &[&str], timeout: Duration) -> Result<String> {
        let mut child = Command::new("/usr/bin/powermetrics")
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ThermalError::Subprocess(format!("powermetrics spawn: {e}")))?;

        let start = Instant::now();
        loop {
            if let Some(status) =
                child.try_wait().map_err(|e| ThermalError::Subprocess(format!("try_wait: {e}")))?
            {
                let mut out = Vec::new();
                if let Some(mut s) = child.stdout.take() {
                    let _ = s.read_to_end(&mut out);
                }
                if !status.success() {
                    return Err(ThermalError::Subprocess(format!(
                        "powermetrics exit={}",
                        status.code().unwrap_or(-1)
                    )));
                }
                return Ok(String::from_utf8_lossy(&out).to_string());
            }

            if start.elapsed() >= timeout {
                let _ = child.kill();
                let _ = child.wait();
                return Err(ThermalError::Subprocess(format!("powermetrics timeout after {:?}", timeout)));
            }

            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

/// Parse `powermetrics` text (samplers `thermal` or `smc`) and extract best-effort temps.
pub fn parse_powermetrics_thermal(s: &str) -> ThermalSnapshot {
    let mut snap = ThermalSnapshot::default();

    // CPU/GPU/SoC die temperature patterns.
    let re_cpu = Regex::new(r"(?mi)^\s*CPU (?:die )?temperature:\s*([\d\.]+)\s*C").unwrap();
    let re_gpu = Regex::new(r"(?mi)^\s*GPU (?:die )?temperature:\s*([\d\.]+)\s*C").unwrap();
    let re_soc = Regex::new(r"(?mi)^\s*(?:SoC|Package) (?:die )?temperature:\s*([\d\.]+)\s*C").unwrap();

    if let Some(c) = re_cpu.captures(s) {
        if let Ok(v) = c.get(1).unwrap().as_str().parse::<f32>() {
            snap.cpu_temp_c = Some(v);
        }
    }
    if let Some(c) = re_gpu.captures(s) {
        if let Ok(v) = c.get(1).unwrap().as_str().parse::<f32>() {
            snap.gpu_temp_c = Some(v);
        }
    }
    if let Some(c) = re_soc.captures(s) {
        if let Ok(v) = c.get(1).unwrap().as_str().parse::<f32>() {
            snap.soc_temp_c = Some(v);
        }
    }

    // Throttling / thermal level (best effort).
    let re_level_num = Regex::new(r"(?mi)CPU Thermal level:\s*(\d+)").unwrap();
    let re_level_txt = Regex::new(r"(?mi)CPU Thermal level:\s*(Nominal|Warning|Serious|Critical)").unwrap();

    if let Some(c) = re_level_num.captures(s) {
        if let Ok(n) = c.get(1).unwrap().as_str().parse::<u32>() {
            snap.throttling = Some(n > 0);
        }
    } else if let Some(c) = re_level_txt.captures(s) {
        let lvl = c.get(1).unwrap().as_str();
        snap.throttling = Some(!matches!(lvl, "Nominal" | "nominal"));
    }

    snap
}