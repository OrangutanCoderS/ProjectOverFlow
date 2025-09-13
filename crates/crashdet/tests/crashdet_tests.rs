use crashdet::{CrashConfig, poll_crashes};
use tempfile::tempdir;
use std::fs;
use std::io::Write;

#[test]
fn returns_empty_or_vec_not_error() {
    let cfg = CrashConfig::default();
    let r = poll_crashes(&cfg);
    assert!(r.is_ok());
    let v = r.unwrap();
    assert!(v.len() <= cfg.max_entries_per_sample);
}

#[test]
fn parses_app_crash_file_basic() {
    let td = tempdir().unwrap();
    let path = td.path().join("Spotify.crash");
    let mut f = fs::File::create(&path).unwrap();
    // Minimal realistic .crash snippet
    writeln!(f, "Process:\tSpotify [4123]").unwrap();
    writeln!(f, "Exception Type:\tEXC_BAD_ACCESS (SIGSEGV)").unwrap();
    drop(f);

    let mut cfg = CrashConfig::default();
    cfg.crash_dirs = vec![td.path().to_path_buf()];
    cfg.max_entries_per_sample = 10;

    let out = poll_crashes(&cfg).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].process, "Spotify");
    assert_eq!(out[0].pid, Some(4123));
    assert!(out[0].crash_reason.contains("EXC_BAD_ACCESS"));
    assert_eq!(out[0].severity, "app_crash");
}

#[test]
fn parses_kernel_panic_log_basic() {
    let td = tempdir().unwrap();
    let path = td.path().join("panic.log");
    let mut f = fs::File::create(&path).unwrap();
    writeln!(f, "panic(cpu 0 caller 0xffff): kernel panic: Aiee!").unwrap();
    drop(f);

    let mut cfg = CrashConfig::default();
    cfg.crash_dirs = vec![td.path().to_path_buf()];
    cfg.max_entries_per_sample = 10;

    let out = poll_crashes(&cfg).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].process, "kernel");
    assert_eq!(out[0].pid, None);
    assert!(out[0].crash_reason.to_lowercase().contains("panic"));
    assert_eq!(out[0].severity, "panic");
}
