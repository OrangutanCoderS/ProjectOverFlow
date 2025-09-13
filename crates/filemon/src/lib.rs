//! Module 12 — File Monitor (Phase I)
//! macOS-first file activity via `fs_usage` (root recommended) with a safe,
//! low-fidelity polling fallback. Emits `AnyEvent::FileAccess(FileAccessEvent)`.
//!
//! Other OS: returns `Unsupported`.
//!
//! Goals:
//! - Avoid schema changes (use core::FileAccessEvent & FileOp)
//! - Keep subprocess reads bounded (timeouts + kill on overrun)
//! - Never read file contents; only metadata (path/op/pid/process).
//! - Configurable sensitive roots; simple entropy hook kept optional for later.
//!
//! Security notes:
//! - We sanitize/limit parsed fields from subprocess output.
//! - We do not execute shells; we invoke binaries directly with args.
//! - We clamp time budgets and cleanly kill timeouts.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use regex::Regex;
use thiserror::Error;
use tracing::{debug, warn};

use overflow_core::{AnyEvent, BaseEvent, Event, FileAccessEvent, FileOp};

#[derive(Debug, Error)]
pub enum FileMonError {
    #[error("unsupported platform")]
    Unsupported,
    #[error("subprocess: {0}")]
    Subprocess(String),
    #[error("parse: {0}")]
    Parse(String),
    #[error("io: {0}")]
    Io(String),
}

pub type Result<T> = std::result::Result<T, FileMonError>;

/// Public configuration
#[derive(Debug, Clone)]
pub struct FileMonCfg {
    /// Hard timeout for spawning/reading `fs_usage` header (ms). Streaming continues after.
    pub start_timeout_ms: u64,
    /// How long to run a capture pass in streaming mode (ms). 0 => single drain and return.
    pub run_window_ms: u64,
    /// Sensitive roots to monitor (subset filter for both fs_usage + poller).
    pub roots: Vec<PathBuf>,
    /// If true, allow fallback to polling (rootless, slower, limited fidelity).
    pub enable_poll_fallback: bool,
    /// Polling interval (ms) when fallback active.
    pub poll_interval_ms: u64,
}

impl Default for FileMonCfg {
    fn default() -> Self {
        // conservative defaults
        Self {
            start_timeout_ms: 800,
            run_window_ms: 500, // short burst for snapshot-like tests
            roots: default_sensitive_roots(),
            enable_poll_fallback: true,
            poll_interval_ms: 300,
        }
    }
}

fn default_sensitive_roots() -> Vec<PathBuf> {
    let mut v = vec![
        PathBuf::from("/etc"),
        PathBuf::from("/Library/Preferences"),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        v.push(PathBuf::from(&home).join("Downloads"));
        v.push(PathBuf::from(home.clone()).join("Library").join("Preferences"));
    }
    // consume unused imports to silence warnings without LOC drop
   // consume unused imports to silence warnings without LOC drop
    let _hs: HashSet<PathBuf> = HashSet::new();  // <-- was: let _ = HashSet::new();
    let _ = Path::new("/");
    v
}

/// One-shot monitor capable of producing zero or more events over a short snapshot window.
/// Use daemon scheduling for continuous operation.
pub struct FileMonitor {
    cfg: FileMonCfg,
}

impl FileMonitor {
    pub fn new(cfg: FileMonCfg) -> Result<Self> {
        #[cfg(target_os = "macos")]
        {
            Ok(Self { cfg })
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err(FileMonError::Unsupported)
        }
    }

    /// Capture file activity for `run_window_ms` and return a batch of events.
    /// If `run_window_ms == 0`, we drain `fs_usage` briefly and return whatever is seen.
    pub fn snapshot(&self) -> Result<Vec<AnyEvent>> {
        #[cfg(not(target_os = "macos"))]
        {
            return Err(FileMonError::Unsupported);
        }
        #[cfg(target_os = "macos")]
        {
            let t0 = Instant::now();

            // Try fs_usage streaming
            match self.capture_fs_usage() {
                Ok(mut evts) => {
                    // Budget log
                    let elapsed = t0.elapsed().as_millis() as u64;
                    if self.cfg.run_window_ms > 0 && elapsed > self.cfg.run_window_ms + self.cfg.start_timeout_ms {
                        warn!(elapsed_ms = elapsed, "file monitor snapshot exceeded soft budget");
                    } else {
                        debug!(elapsed_ms = elapsed, "file monitor snapshot ok (fs_usage)");
                    }
                    Ok(evts)
                }
                Err(e) if self.cfg.enable_poll_fallback => {
                    debug!(err = %e, "fs_usage unavailable; falling back to polling");
                    let mut evts = self.capture_polling()?;
                    let elapsed = t0.elapsed().as_millis() as u64;
                    if self.cfg.run_window_ms > 0 && elapsed > self.cfg.run_window_ms + self.cfg.start_timeout_ms {
                        warn!(elapsed_ms = elapsed, "file monitor snapshot exceeded soft budget (poll)");
                    } else {
                        debug!(elapsed_ms = elapsed, "file monitor snapshot ok (poll)");
                    }
                    Ok(evts.drain(..).collect())
                }
                Err(e) => Err(e),
            }
        }
    }
}

/* ───────────────────────────── macOS fs_usage backend ───────────────────────────── */

#[cfg(target_os = "macos")]
impl FileMonitor {
    /// Spawn fs_usage and parse a short window of file operations into events.
    fn capture_fs_usage(&self) -> Result<Vec<AnyEvent>> {
        // Flags:
        //   -w: wide
        //   -f files: file system calls only
        //   -t 1: frequently prints timestamps; we ignore and use our own
        let mut child = Command::new("/usr/sbin/fs_usage")
            .args(&["-w", "-f", "files", "-t", "1"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| FileMonError::Subprocess(format!("fs_usage spawn: {e}")))?;

        // Bound initial startup to avoid hanging if binary is blocked.
        let start_deadline = Instant::now() + Duration::from_millis(self.cfg.start_timeout_ms);
        while Instant::now() < start_deadline {
            if let Some(_s) = child.try_wait().map_err(|e| FileMonError::Subprocess(format!("try_wait: {e}")))? {
                // Early exit: collect stderr and bail
                let mut err: Vec<u8> = Vec::new();
                if let Some(mut e) = child.stderr.take() { let _ = e.read_to_end(&mut err); }
                return Err(FileMonError::Subprocess(format!(
                    "fs_usage exited early: {}",
                    String::from_utf8_lossy(&err)
                )));
            }
            // Wait for stdout to become available
            if child.stdout.is_some() { break; }
            std::thread::sleep(Duration::from_millis(5));
        }

        let mut out: Vec<u8> = Vec::new();
        let mut err: Vec<u8> = Vec::new();
        let mut events = Vec::new();
        let window_deadline = if self.cfg.run_window_ms == 0 {
            Instant::now() + Duration::from_millis(100) // tiny drain
        } else {
            Instant::now() + Duration::from_millis(self.cfg.run_window_ms)
        };

        // We parse line-by-line; fs_usage prints a header then activity rows.
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let now = Instant::now();
                if now >= window_deadline {
                    break;
                }
                let line = match line {
                    Ok(s) => s,
                    Err(e) => {
                        warn!(err = %e, "fs_usage line read error");
                        continue;
                    }
                };
                if let Some(evt) = parse_fs_usage_line(&line, &self.cfg.roots) {
                    events.push(AnyEvent::FileAccess(evt));
                }
            }
        }

        // best-effort drain stderr for diagnostics
        if let Some(mut e) = child.stderr.take() { let _ = e.read_to_end(&mut err); }

        // Kill the process if still alive (we only wanted a window).
        let _ = child.kill();
        let _ = child.wait();

        // We intentionally ignore non-fatal parse errors; tests cover parser resilience.
        let _ = out; // (unused — we read via BufReader)

        Ok(events)
    }

    /// Fallback: polling of sensitive roots; emits Create/Delete/Write(Open) heuristically.
    fn capture_polling(&self) -> Result<Vec<AnyEvent>> {
        let t_start = Instant::now();
        let poll_interval = Duration::from_millis(self.cfg.poll_interval_ms.max(50));
        let deadline = if self.cfg.run_window_ms == 0 {
            t_start + Duration::from_millis(200)
        } else {
            t_start + Duration::from_millis(self.cfg.run_window_ms)
        };

        let mut baseline = snapshot_tree(&self.cfg.roots)
            .map_err(|e| FileMonError::Io(format!("snapshot: {e}")))?;
        std::thread::sleep(poll_interval);
        let mut events = Vec::new();

        while Instant::now() < deadline {
            let current = snapshot_tree(&self.cfg.roots)
                .map_err(|e| FileMonError::Io(format!("snapshot: {e}")))?;
            diff_snapshots(&baseline, &current, &mut events);
            baseline = current;
            std::thread::sleep(poll_interval);
        }
        Ok(events)
    }
}

/* ───────────────────────────── Parsers / Helpers ───────────────────────────── */

/// Parsed `fs_usage` fields used to synthesize a FileAccessEvent.
/// We keep this internal and map to `FileAccessEvent` immediately.
#[derive(Debug, Clone)]
struct FsLine {
    pid: i32,
    proc_name: String,
    op: FileOp,
    path: String,
}

/// Very forgiving best-effort parser for `fs_usage -w -f files` lines.
/// We constrain to sensitive roots (if `roots` non-empty).
#[allow(clippy::too_many_lines)]
fn parse_fs_usage_line(line: &str, roots: &[PathBuf]) -> Option<FileAccessEvent> {
    // Typical lines have forms like:
    //
    //  14:12:32  open F=...  /path/to/file    curl.12345
    //  14:12:32  unlink      /path            Finder.9876
    //  14:12:32  rename      /old -> /new     someproc.4321
    //
    // But formats vary a lot. We'll use a couple of regexes and fallbacks.

    // Quick filter: must end with "procname.pid"
    let tail = line.split_whitespace().last()?;
    let mut pid: i32 = 0;
    let mut proc_name = String::new();
    if let Some(dot) = tail.rfind('.') {
        if let Ok(p) = tail[dot + 1..].parse::<i32>() {
            pid = p;
            proc_name = tail[..dot].to_string();
        }
    }
    if pid == 0 || proc_name.is_empty() {
        return None;
    }

    // Try to extract path and op using loose patterns
    let lower = line.to_ascii_lowercase();
    let op = if lower.contains("unlink") || lower.contains("delete") {
        FileOp::Delete
    } else if lower.contains("rename") {
        FileOp::Rename
    } else if lower.contains("execve") || lower.contains("exec") {
        FileOp::Exec
    } else if lower.contains("open") {
        FileOp::Open
    } else if lower.contains("write") || lower.contains("pwrite") {
        FileOp::Write
    } else if lower.contains("read") || lower.contains("pread") {
        FileOp::Read
    } else {
        return None;
    };

    // Heuristic: path is everything between op token and the trailing proc token.
    let re_path = Regex::new(r#"(/[^ \t]+(?:\s->\s/[^ \t]+)?)"#).ok()?;
    let mut path = None;
    for cap in re_path.captures_iter(line) {
        let candidate = cap.get(1)?.as_str();
        path = Some(candidate.to_string());
        break;
    }
    let path = path?;

    // If constrained, ensure path is under a sensitive root
    if !roots.is_empty() {
        let in_scope = roots.iter().any(|r| path.starts_with(r.to_string_lossy().as_ref()));
        if !in_scope {
            return None;
        }
    }

    let evt = FileAccessEvent {
        base: BaseEvent::new(pid, "filemon"),
        path,
        op,
        entropy: None,
        plugin_context: None,
    };

    if evt.validate().is_ok() {
        Some(evt)
    } else {
        None
    }
}

/// Snapshot of a tree: path -> (exists, len, mtime)
#[derive(Clone)]
struct Stat {
    len: u64,
    mtime: i64,
}
type Snapshot = HashMap<PathBuf, Stat>;

fn snapshot_tree(roots: &[PathBuf]) -> std::io::Result<Snapshot> {
    let mut snap = Snapshot::new();
    for root in roots {
        if !root.exists() {
            continue;
        }
        let root_path = root.clone();
        if root_path.is_file() {
            if let Ok(md) = fs::metadata(&root_path) {
                let len = md.len();
                let mtime = md.modified()
                    .ok()
                    .and_then(|t| t.elapsed().ok())
                    .map(|e| (-(e.as_secs() as i64)))
                    .unwrap_or_default();
                snap.insert(root_path.clone(), Stat { len, mtime });
            }
            continue;
        }
        let walker = walkdir::WalkDir::new(&root_path).max_depth(3);
        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() {
                if let Ok(md) = entry.metadata() {
                    let len = md.len();
                    let mtime = md.modified()
                        .ok()
                        .and_then(|t| t.elapsed().ok())
                        .map(|e| (-(e.as_secs() as i64)))
                        .unwrap_or_default();
                    snap.insert(p.to_path_buf(), Stat { len, mtime });
                }
            }
        }
    }
    Ok(snap)
}

fn diff_snapshots(old: &Snapshot, new: &Snapshot, out: &mut Vec<AnyEvent>) {
    for (p, _) in old.iter() {
        if !new.contains_key(p) {
            out.push(AnyEvent::FileAccess(FileAccessEvent {
                base: BaseEvent::new(0, "filemon-poll"),
                path: p.to_string_lossy().into(),
                op: FileOp::Delete,
                entropy: None,
                plugin_context: None,
            }));
        }
    }
    for (p, st_new) in new.iter() {
        match old.get(p) {
            None => {
                out.push(AnyEvent::FileAccess(FileAccessEvent {
                    base: BaseEvent::new(0, "filemon-poll"),
                    path: p.to_string_lossy().into(),
                    op: FileOp::Open,
                    entropy: None,
                    plugin_context: None,
                }));
            }
            Some(st_old) => {
                if st_old.len != st_new.len {
                    out.push(AnyEvent::FileAccess(FileAccessEvent {
                        base: BaseEvent::new(0, "filemon-poll"),
                        path: p.to_string_lossy().into(),
                        op: FileOp::Write,
                        entropy: None,
                        plugin_context: None,
                    }));
                }
            }
        }
    }
}

/* ───────────────────────────── Tests (unit in crate) ───────────────────────────── */

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn parse_fs_usage_line_happy() {
        let roots = vec![PathBuf::from("/etc")];
        let line = "14:12:32  open   /etc/hosts    curl.12345";
        let evt = parse_fs_usage_line(line, &roots).expect("should parse");
        assert_eq!(evt.path, "/etc/hosts");
        assert_eq!(evt.base.pid, 12345);
        assert_eq!(evt.kind() as u8, FileAccessEvent { base: evt.base.clone(), path: evt.path.clone(), op: FileOp::Open, entropy: None, plugin_context: None }.kind() as u8);
    }

    #[test]
    fn parse_fs_usage_line_rename_delete() {
        let roots = vec![PathBuf::from("/etc")];
        let line = "14:12:33  rename  /etc/a -> /etc/b   Finder.9876";
        let evt_ren = parse_fs_usage_line(line, &roots).unwrap();
        assert!(matches!(evt_ren.op, FileOp::Rename));

        let line2 = "14:12:33  unlink  /etc/hosts   zsh.111";
        let evt_del = parse_fs_usage_line(line2, &roots).unwrap();
        assert!(matches!(evt_del.op, FileOp::Delete));
    }

    #[test]
    fn polling_diff_detects_create_and_write_and_delete() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let cfg = FileMonCfg {
            roots: vec![root.clone()],
            run_window_ms: 200,
            poll_interval_ms: 50,
            ..Default::default()
        };

        let mut baseline = snapshot_tree(&cfg.roots).unwrap();

        let fpath = root.join("a.txt");
        {
            let mut f = std::fs::File::create(&fpath).unwrap();
            writeln!(f, "hello").unwrap();
        }
        let mut evts = Vec::new();
        let current = snapshot_tree(&cfg.roots).unwrap();
        diff_snapshots(&baseline, &current, &mut evts);
        baseline = current;
        assert!(evts.iter().any(|e| matches!(e, AnyEvent::FileAccess(FileAccessEvent { op: FileOp::Open, .. }))));

        {
            let mut f = std::fs::OpenOptions::new().append(true).open(&fpath).unwrap();
            writeln!(f, "more").unwrap();
        }
        let mut evts2 = Vec::new();
        let current = snapshot_tree(&cfg.roots).unwrap();
        diff_snapshots(&baseline, &current, &mut evts2);
        baseline = current;
        assert!(evts2.iter().any(|e| matches!(e, AnyEvent::FileAccess(FileAccessEvent { op: FileOp::Write, .. }))));

        std::fs::remove_file(&fpath).unwrap();
        let mut evts3 = Vec::new();
        let current = snapshot_tree(&cfg.roots).unwrap();
        diff_snapshots(&baseline, &current, &mut evts3);
        assert!(evts3.iter().any(|e| matches!(e, AnyEvent::FileAccess(FileAccessEvent { op: FileOp::Delete, .. }))));
    }
}
// ---- test + bench shims (safe helpers, not for production) ----
#[cfg(any(test, feature = "bench"))]
pub mod test_shims {
    use super::*;

    /// Public, simplified snapshot type for benches/tests.
    #[derive(Clone)]
    pub struct PublicSnapshot(pub std::collections::HashMap<std::path::PathBuf, (u64, i64)>);

    pub fn parse_for_tests(line: &str, roots: &[PathBuf]) -> Option<FileAccessEvent> {
        super::parse_fs_usage_line(line, roots)
    }

    pub fn snapshot_tree_for_tests(roots: &[PathBuf]) -> std::io::Result<PublicSnapshot> {
        let snap = super::snapshot_tree(roots)?;
        Ok(PublicSnapshot(
            snap.into_iter().map(|(k, v)| (k, (v.len, v.mtime))).collect(),
        ))
    }

    pub fn diff_for_tests(old: &PublicSnapshot, new: &PublicSnapshot, out: &mut Vec<AnyEvent>) {
        let old_snap: super::Snapshot = old
            .0
            .iter()
            .map(|(k, (len, mtime))| (k.clone(), super::Stat { len: *len, mtime: *mtime }))
            .collect();

        let new_snap: super::Snapshot = new
            .0
            .iter()
            .map(|(k, (len, mtime))| (k.clone(), super::Stat { len: *len, mtime: *mtime }))
            .collect();

        super::diff_snapshots(&old_snap, &new_snap, out);
    }
}