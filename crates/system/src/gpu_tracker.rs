//! Module 9 — GPU Tracker
//! macOS-first GPU snapshot using `powermetrics` (root) with a safe fallback
//! to `system_profiler` (rootless). Returns `AnyEvent::Gpu(GpuEvent)`.

use regex::Regex;
use serde_json::Value as Json;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::debug;

use overflow_core::{AnyEvent, BaseEvent, Event, GpuEvent};

#[derive(Debug, Error)]
pub enum GpuError {
    #[error("unsupported platform")]
    Unsupported,
    #[error("subprocess: {0}")]
    Subprocess(String),
    #[error("parse: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, GpuError>;

#[derive(Debug, Clone, Copy)]
pub struct GpuCfg {
    /// Hard timeout for powermetrics/system_profiler calls.
    pub timeout_ms: u64,
    /// Prefer powermetrics (more accurate) if available.
    pub prefer_powermetrics: bool,
}

impl Default for GpuCfg {
    fn default() -> Self {
        Self {
            timeout_ms: 800,
            prefer_powermetrics: true,
        }
    }
}

pub struct GpuTracker {
    cfg: GpuCfg,
}

impl GpuTracker {
    pub fn new(cfg: GpuCfg) -> Result<Self> {
        #[cfg(target_os = "macos")]
        {
            Ok(Self { cfg })
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err(GpuError::Unsupported)
        }
    }

    /// Capture one GPU snapshot and return as AnyEvent::Gpu.
    pub fn capture(&self) -> Result<AnyEvent> {
        #[cfg(not(target_os = "macos"))]
        {
            return Err(GpuError::Unsupported);
        }

        #[cfg(target_os = "macos")]
        {
            let t0 = Instant::now();
            let timeout = Duration::from_millis(self.cfg.timeout_ms);

            // Try powermetrics first (if desired), else fallback.
            let fields = if self.cfg.prefer_powermetrics {
                self.sample_powermetrics(timeout).or_else(|e| {
                    debug!(err = %e, "powermetrics failed; falling back to system_profiler");
                    self.sample_system_profiler(timeout)
                })?
            } else {
                match self.sample_system_profiler(timeout) {
                    Ok(f) => f,
                    Err(e1) => {
                        debug!(err = %e1, "system_profiler failed; trying powermetrics");
                        self.sample_powermetrics(timeout)?
                    }
                }
            };

            let evt = GpuEvent {
                base: BaseEvent::new(0, "gpu_tracker"),
                name: fields.model.unwrap_or_else(|| "Unknown GPU".into()),
                usage_pct: fields.usage_pct.unwrap_or(0.0),
                temperature_c: fields.temperature_c.unwrap_or(0.0),
                mem_total_mb: fields.memory_total_mb.unwrap_or(0.0),
                mem_used_mb: fields.memory_used_mb.unwrap_or(0.0),
            };

            // Validate to keep models tight.
            if let Err(e) = evt.validate() {
                return Err(GpuError::Parse(e.to_string()));
            }

            let elapsed = t0.elapsed().as_millis() as u64;
            debug!(elapsed_ms = elapsed, "gpu snapshot collected");
            Ok(AnyEvent::Gpu(evt))
        }
    }
}

/* ---------- sampling + parsing (macOS) ---------- */

#[derive(Debug, Default)]
struct Fields {
    model: Option<String>,
    usage_pct: Option<f32>,
    temperature_c: Option<f32>,
    memory_total_mb: Option<f32>, // VRAM capacity
    memory_used_mb: Option<f32>,  // used VRAM if available
}

impl GpuTracker {
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
            .map_err(|e| GpuError::Subprocess(format!("{program} spawn: {e}")))?;

        let start = Instant::now();
        loop {
            if let Some(status) = child
                .try_wait()
                .map_err(|e| GpuError::Subprocess(format!("try_wait: {e}")))? 
            {
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
                return Err(GpuError::Subprocess(format!(
                    "{program} timed out after {:?}. stderr: {}",
                    timeout,
                    String::from_utf8_lossy(&err)
                )));
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// Prefer powermetrics (requires root; otherwise may error or return partials).
    #[cfg(target_os = "macos")]
    fn sample_powermetrics(&self, timeout: Duration) -> Result<Fields> {
        let args = ["-n", "1", "-i", "100", "--samplers", "gpu_power"];
        let (code, out, err) = self.run_with_timeout("/usr/bin/powermetrics", &args, timeout)?;
        if code != 0 {
            return Err(GpuError::Subprocess(format!(
                "powermetrics exit={code}, stderr={}",
                String::from_utf8_lossy(&err)
            )));
        }
        let s = String::from_utf8_lossy(&out);
        Ok(parse_powermetrics(&s))
    }

    /// Fallback: `system_profiler SPDisplaysDataType -json`
    #[cfg(target_os = "macos")]
    fn sample_system_profiler(&self, timeout: Duration) -> Result<Fields> {
        let (code, out, err) = self.run_with_timeout(
            "/usr/sbin/system_profiler",
            &["SPDisplaysDataType", "-json"],
            timeout,
        )?;
        if code != 0 {
            return Err(GpuError::Subprocess(format!(
                "system_profiler exit={code}, stderr={}",
                String::from_utf8_lossy(&err)
            )));
        }
        let s = String::from_utf8_lossy(&out);
        Ok(parse_system_profiler_json(&s))
    }
}

/* ---------- parsers (unit-tested, resilient) ---------- */

/// Parse powermetrics text for GPU fields (best-effort).
pub(crate) fn parse_powermetrics(s: &str) -> Fields {
    let mut f = Fields::default();

    let re_temp =
        Regex::new(r"(?m)^\s*GPU (?:die )?temperature:\s*([\d\.]+)\s*C").unwrap();
    let re_usage = Regex::new(r"(?m)^\s*graphics:\s*([\d\.]+)%").unwrap();
    let re_usage2 = Regex::new(r"(?m)^\s*GPU .*?([\d\.]+)\s*%").unwrap();

    if let Some(c) = re_temp.captures(s) {
        if let Ok(v) = c.get(1).unwrap().as_str().parse::<f32>() {
            f.temperature_c = Some(v);
        }
    }
    if let Some(c) = re_usage.captures(s) {
        if let Ok(v) = c.get(1).unwrap().as_str().parse::<f32>() {
            f.usage_pct = Some(v);
        }
    } else if let Some(c) = re_usage2.captures(s) {
        if let Ok(v) = c.get(1).unwrap().as_str().parse::<f32>() {
            f.usage_pct = Some(v);
        }
    }

    f
}

/// Parse `system_profiler ... -json` for GPU model & VRAM.
pub(crate) fn parse_system_profiler_json(s: &str) -> Fields {
    let mut f = Fields::default();
    if let Ok(json) = serde_json::from_str::<Json>(s) {
        if let Some(arr) = json
            .get("SPDisplaysDataType")
            .and_then(|v| v.as_array())
        {
            if let Some(first) = arr.first() {
                if let Some(m) = first.get("sppci_model").and_then(|v| v.as_str()) {
                    f.model = Some(m.to_string());
                }
                if let Some(vram) = first
                    .get("spdisplays_vram")
                    .or_else(|| first.get("sppci_vram"))
                    .and_then(|v| v.as_str())
                {
                    if let Some(num) = vram.split_whitespace().next() {
                        if let Ok(mb) = num.replace(',', "").parse::<f32>() {
                            f.memory_total_mb = Some(mb);
                        }
                    }
                }
            }
        }
    }
    f
}

/* ---------------------- Tests ---------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_powermetrics_happy() {
        let sample = r#"
==== GPU Stats ====
GPU die temperature: 37.6 C
graphics: 25%
"#;
        let f = parse_powermetrics(sample);
        assert_eq!(f.temperature_c, Some(37.6));
        assert_eq!(f.usage_pct, Some(25.0));
    }

    #[test]
    fn parse_system_profiler_basic() {
        let json = r#"{
          "SPDisplaysDataType" : [
            { "sppci_model": "Apple M2", "spdisplays_vram": "1536 MB" }
          ]
        }"#;
        let f = parse_system_profiler_json(json);
        assert_eq!(f.model.as_deref(), Some("Apple M2"));
        assert_eq!(f.memory_total_mb, Some(1536.0));
    }
}