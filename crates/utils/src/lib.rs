//! Module 4 — Utilities (Phase I)
//! Small, safe, reusable helpers with zero side effects across the rest of the system.

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::{
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

/// --- Time helpers ---

/// Returns current UTC as ISO-8601 string (`YYYY-MM-DDTHH:MM:SS.mmmZ`)
pub fn utc_iso8601() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// Formats a UNIX ms timestamp into a human string ("YYYY-MM-DD HH:MM:SS UTC")
pub fn fmt_ts_ms_human(ms: i64) -> String {
    let dt: DateTime<Utc> =
        DateTime::<Utc>::from(std::time::UNIX_EPOCH + Duration::from_millis(ms as u64));
    format!("{} UTC", dt.format("%Y-%m-%d %H:%M:%S"))
}

/// --- Size/bytes helpers ---

/// Formats bytes using 1024 base (KiB/MiB/GiB), e.g. `10.5 MiB`
pub fn fmt_bytes_iec(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }

    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = bytes as f64;
    let mut idx = 0usize;
    while size >= 1024.0 && idx < UNITS.len() - 1 {
        size /= 1024.0;
        idx += 1;
    }

    // For whole numbers, drop decimal point
    if (size - size.round()).abs() < f64::EPSILON {
        format!("{} {}", size as u64, UNITS[idx])
    } else {
        format!("{:.1} {}", size, UNITS[idx])
    }
}

/// --- Safe subprocess execution ---

/// Result of a subprocess execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcOutput {
    pub status_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub duration_ms: u128,
}

/// Executes a program with args safely:
/// - No shell invocation (prevents injection)
/// - Captures stdout/stderr
/// - Enforces `timeout` (kills the process if exceeded)
/// - Returns exit code (-1 if terminated)
pub fn run_command_with_timeout<I, S>(
    program: &str,
    args: I,
    timeout: Duration,
) -> io::Result<ProcOutput>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let start = Instant::now();
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // poll loop for cross-platform timeout (std only)
    loop {
        if let Some(status) = child.try_wait()? {
            let mut out = Vec::new();
            let mut err = Vec::new();
            if let Some(mut s) = child.stdout.take() {
                s.read_to_end(&mut out)?;
            }
            if let Some(mut s) = child.stderr.take() {
                s.read_to_end(&mut err)?;
            }
            return Ok(ProcOutput {
                status_code: status.code().unwrap_or(-1),
                stdout: out,
                stderr: err,
                duration_ms: start.elapsed().as_millis(),
            });
        }
        if start.elapsed() >= timeout {
            // best effort kill + drain pipes
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
            return Ok(ProcOutput {
                status_code: -1,
                stdout: out,
                stderr: err,
                duration_ms: start.elapsed().as_millis(),
            });
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// --- Entropy (Shannon) ---

/// Shannon entropy (bits/byte) for a byte slice (0.0..=8.0)
pub fn shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut freq = [0u32; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let len = data.len() as f64;
    let mut h = 0.0f64;
    for &c in &freq {
        if c == 0 {
            continue;
        }
        let p = c as f64 / len;
        h -= p * p.log2();
    }
    h
}

/// Reads up to `max_bytes` from `path` and computes entropy.
pub fn file_entropy_limited<P: AsRef<Path>>(path: P, max_bytes: usize) -> io::Result<f64> {
    let mut f = File::open(path)?;
    let mut buf = Vec::with_capacity(max_bytes.min(1 << 20));
    // Disambiguate by_ref to the Read trait to avoid E0034
    std::io::Read::by_ref(&mut f)
        .take(max_bytes as u64)
        .read_to_end(&mut buf)?;
    Ok(shannon_entropy(&buf))
}

/// --- Atomic JSON writer with path safety ---

/// Ensures `target` is inside `base_dir` (canonical path check).
pub fn sanitize_path_under(base_dir: &Path, target: &Path) -> io::Result<PathBuf> {
    let base = base_dir.canonicalize()?;
    let full = if target.is_absolute() {
        target.to_path_buf()
    } else {
        base.join(target)
    };
    let canon = full.canonicalize()?;
    if !canon.starts_with(&base) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "path escapes base_dir",
        ));
    }
    Ok(canon)
}

/// Writes JSON atomically with restricted permissions:
/// - serialize to temp file in same dir
/// - fsync
/// - rename over destination
/// - permissions: 0o600 (unix) / normal file (windows)
pub fn write_json_atomic<P: AsRef<Path>, T: Serialize>(dest: P, value: &T) -> io::Result<()> {
    let dest = dest.as_ref();
    let parent = dest
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no parent directory"))?;
    fs::create_dir_all(parent)?;

    let tmp_name = format!(
        ".{}.tmp-{}",
        dest.file_name().and_then(OsStr::to_str).unwrap_or("out"),
        std::process::id()
    );
    let tmp_path = parent.join(tmp_name);

    // write temp
    {
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)?;
        // permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600));
        }
        let buf = serde_json::to_vec(value)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        f.write_all(&buf)?;
        f.flush()?;
        // fsync
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            let _ = nix_like_fsync(f.as_raw_fd());
        }
        #[cfg(windows)]
        {
            let _ = f.sync_all();
        }
    }

    // atomic rename
    fs::rename(&tmp_path, dest)?;
    Ok(())
}

#[cfg(unix)]
fn nix_like_fsync(fd: std::os::fd::RawFd) -> io::Result<()> {
    unsafe {
        let rc = libc::fsync(fd);
        if rc == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

/// --- Small helpers ---

/// Guard for simple positive integer checks (defensive programming).
pub fn ensure_positive_i32(v: i32, name: &str) -> Result<(), String> {
    if v <= 0 {
        Err(format!("{name} must be > 0"))
    } else {
        Ok(())
    }
}

// === Public API aliases for consistency with tests ===
pub fn bytes_human(n: u64) -> String {
    fmt_bytes_iec(n)
}

pub fn unix_time_ms() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn run_cmd(cmd: &str, args: &[&str], timeout_ms: Option<u64>) -> std::io::Result<String> {
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(1000));
    let out = run_command_with_timeout(cmd, args, timeout)?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}
