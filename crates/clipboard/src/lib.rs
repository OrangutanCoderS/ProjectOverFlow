use anyhow::{anyhow, Context, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

// Your hardened helpers (crate name: overflow-utils / lib name: overflow_utils)
use overflow_utils::{run_command_with_timeout, utc_iso8601, ProcOutput};

/// Clipboard event model — consistent with *_info naming.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClipboardEventInfo {
    pub timestamp: String,     // ISO-8601 UTC
    pub event: String,         // "clipboard_update"
    pub content_hash: String,  // hex SHA256
    pub entropy_score: f64,    // Shannon entropy (bytes)
    pub data_type: String,     // "text" | "binary"
    pub length: usize,         // bytes length
    pub source: String,        // "unknown" (Phase I)
    pub content: String,       // Actual clipboard content (if logging plaintext)
}

impl ClipboardEventInfo {
    pub fn validate(&self) -> Result<()> {
        if self.timestamp.is_empty() {
            return Err(anyhow!("timestamp is empty"));
        }
        if self.event != "clipboard_update" {
            return Err(anyhow!("unexpected event kind"));
        }
        if self.content_hash.len() != 64 {
            return Err(anyhow!("sha256 hex length must be 64"));
        }
        if !(self.data_type == "text" || self.data_type == "binary") {
            return Err(anyhow!("invalid data_type"));
        }
        Ok(())
    }
}

/// Module-local config — keeps old files untouched.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardConfig {
    pub poll_interval_ms: u64,        // default 1000ms
    pub log_plaintext: bool,          // default false (hash-only)
    pub entropy_alert_threshold: f64, // e.g., 6.5 (not used here; for plugins)
    pub subprocess_timeout_ms: u64,   // guard pbpaste/xclip/clip.exe
    pub max_capture_bytes: usize,     // safety cap for oversized content
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 1000,
            log_plaintext: true, // Default to true to log clipboard content
            entropy_alert_threshold: 6.5,
            subprocess_timeout_ms: 1500,
            max_capture_bytes: 1024 * 1024, // 1 MiB cap
        }
    }
}

/// Public API: poll once — returns Some(event) only when changed.
pub fn poll_clipboard(cfg: &ClipboardConfig, last_hash: &str) -> Result<Option<ClipboardEventInfo>> {
    let now = Instant::now();
    let data = BACKEND.get_clipboard_bytes(Duration::from_millis(cfg.subprocess_timeout_ms))?;
    let capped = if data.len() > cfg.max_capture_bytes {
        &data[..cfg.max_capture_bytes]
    } else {
        &data[..]
    };

    // Detect type by UTF-8 validation
    let data_type = if std::str::from_utf8(capped).is_ok() { "text" } else { "binary" }.to_string();

    let hash = sha256_hex(capped);
    
    // Only register a new event if the clipboard content has changed
    if hash == last_hash {
        return Ok(None); // unchanged content
    }

    // Update the last_hash after detecting a change
    let entropy = calculate_entropy(capped);
    let content = if cfg.log_plaintext {
        // If plaintext logging is enabled, convert content to a string
        String::from_utf8_lossy(capped).to_string() // Safely convert to string
    } else {
        "".to_string() // If not logging plaintext, leave it empty
    };

    let info = ClipboardEventInfo {
        timestamp: utc_iso8601(),
        event: "clipboard_update".to_string(),
        content_hash: hash,
        entropy_score: entropy,
        data_type,
        length: capped.len(),
        source: "unknown".to_string(),
        content, // Save actual content if logging plaintext
    };

    info.validate()?;

    // Log the elapsed time if needed (for debugging)
    let _elapsed = now.elapsed();

    // Return the updated event
    Ok(Some(info))
}

/// Public API: spawn a background watcher and stream events on change.
pub fn spawn_clipboard_watcher(cfg: ClipboardConfig) -> Receiver<ClipboardEventInfo> {
    let (tx, rx): (Sender<ClipboardEventInfo>, Receiver<ClipboardEventInfo>) = mpsc::channel();
    thread::spawn(move || {
        let mut last_hash = String::new();
        let interval = Duration::from_millis(cfg.poll_interval_ms.max(100)); // Shorter polling interval
        loop {
            match poll_clipboard(&cfg, &last_hash) {
                Ok(Some(ev)) => {
                    last_hash = ev.content_hash.clone(); // Update last_hash after processing the event
                    let _ = tx.send(ev);  // Send event if content has changed
                }
                Ok(None) => { /* No change detected */ }
                Err(_) => { /* Fail gracefully, continue polling */ }
            }
            thread::sleep(interval); // Sleep between polling
        }
    });
    rx
}

/// Calculate Shannon entropy over bytes (per-byte probability).
pub fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut counts = [0usize; 256];
    for b in data {
        counts[*b as usize] += 1;
    }
    let len = data.len() as f64;
    let mut entropy = 0.0;
    for &c in &counts {
        if c == 0 { continue; }
        let p = c as f64 / len;
        entropy -= p * p.log2();
    }
    entropy
}

/// Hex SHA-256 helper (no external string allocations beyond hex).
fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let out = hasher.finalize();
    hex_encode(&out)
}

/// Constant-time-ish hex (tiny data) — keep local to avoid adding deps.
fn hex_encode(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(LUT[(b >> 4) as usize] as char);
        s.push(LUT[(b & 0x0F) as usize] as char);
    }
    s
}

/* ============================
   Backend detection & impls
   ============================ */

trait ClipboardBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn get_clipboard_bytes(&self, timeout: Duration) -> Result<Vec<u8>>;
}

static BACKEND: Lazy<Box<dyn ClipboardBackend>> = Lazy::new(|| {
    // macOS pbpaste
    if MacBackend::available() { return Box::new(MacBackend); }
    // Linux: xclip -> xsel -> wl-paste (best-effort)
    if LinuxBackend::available() { return Box::new(LinuxBackend); }
    // Windows: powershell Get-Clipboard (UTF-8)
    if WindowsBackend::available() { return Box::new(WindowsBackend); }
    Box::new(FallbackBackend)
});

/// macOS implementation using `pbpaste`
/// Rationale: zero extra deps; robust with our timeout guard.
struct MacBackend;
impl MacBackend {
    fn available() -> bool {
        if cfg!(target_os = "macos") {
            return Command::new("which").arg("pbpaste").output().map(|o| o.status.success()).unwrap_or(false);
        }
        false
    }
}
impl ClipboardBackend for MacBackend {
    fn name(&self) -> &'static str { "macos_pbpaste" }

    fn get_clipboard_bytes(&self, timeout: Duration) -> Result<Vec<u8>> {
        let ProcOutput { stdout, .. } = run_command_with_timeout(
            "pbpaste",
            &[] as &[&str],   // 👈 explicit empty slice, fixes E0283
            timeout,
        )
        .context("failed to run pbpaste")?;
        Ok(stdout)
    }
}

/// Linux implementation preferring xclip, falling back to xsel and wl-paste.
struct LinuxBackend;
impl LinuxBackend {
    fn available() -> bool {
        if cfg!(target_os = "linux") {
            let xclip = Command::new("which").arg("xclip").output().map(|o| o.status.success()).unwrap_or(false);
            let xsel  = Command::new("which").arg("xsel").output().map(|o| o.status.success()).unwrap_or(false);
            let wl    = Command::new("which").arg("wl-paste").output().map(|o| o.status.success()).unwrap_or(false);
            return xclip || xsel || wl;
        }
        false
    }
}
impl ClipboardBackend for LinuxBackend {
    fn name(&self) -> &'static str { "linux_clipboard" }
    fn get_clipboard_bytes(&self, timeout: Duration) -> Result<Vec<u8>> {
        // Try xclip
        if Command::new("which").arg("xclip").output().map(|o| o.status.success()).unwrap_or(false) {
            let out: ProcOutput = run_command_with_timeout("xclip", ["-selection", "clipboard", "-o"], timeout)
                .context("failed to run xclip")?;
            return Ok(out.stdout);
        }
        // Try xsel
        if Command::new("which").arg("xsel").output().map(|o| o.status.success()).unwrap_or(false) {
            let out: ProcOutput = run_command_with_timeout("xsel", ["--clipboard", "--output"], timeout)
                .context("failed to run xsel")?;
            return Ok(out.stdout);
        }
        // Try wl-paste (Wayland)
        if Command::new("which").arg("wl-paste").output().map(|o| o.status.success()).unwrap_or(false) {
            let out: ProcOutput = run_command_with_timeout("wl-paste", ["-n"], timeout)
                .context("failed to run wl-paste")?;
            return Ok(out.stdout);
        }
        Err(anyhow!("no clipboard tool available on Linux"))
    }
}

/// Windows implementation using PowerShell Get-Clipboard (UTF8 out).
struct WindowsBackend;
impl WindowsBackend {
    fn available() -> bool {
        if cfg!(target_os = "windows") {
            return Command::new("where").arg("powershell").output().map(|o| o.status.success()).unwrap_or(false);
        }
        false
    }
}
impl ClipboardBackend for WindowsBackend {
    fn name(&self) -> &'static str { "windows_powershell" }
    fn get_clipboard_bytes(&self, timeout: Duration) -> Result<Vec<u8>> {
        // Force UTF-8 output
        let out: ProcOutput = run_command_with_timeout(
            "powershell",
            ["-NoProfile", "-Command", "Get-Clipboard | Out-String"],
            timeout,
        ).context("failed to run PowerShell Get-Clipboard")?;
        Ok(out.stdout) // bytes (likely UTF-8)
    }
}

/// Fallback returns empty clipboard (compiles everywhere).
struct FallbackBackend;
impl ClipboardBackend for FallbackBackend {
    fn name(&self) -> &'static str { "fallback" }
    fn get_clipboard_bytes(&self, _timeout: Duration) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }
}

/* ============================
   Error surface (public)
   ============================ */
#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error("backend unavailable")]
    BackendUnavailable,
    #[error("parse error: {0}")]
    Parse(String),
}