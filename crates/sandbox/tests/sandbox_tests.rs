use sandbox::{SandboxConfig, run_in_sandbox};

#[test]
fn sandbox_runs_or_stubs() {
    let mut cfg = SandboxConfig::default();

    // Override profile path with absolute path resolved at compile-time
    cfg.profile_path = std::path::PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/profiles/minimal.sb"
    ));

    let res = run_in_sandbox(&cfg, "echo", &["hello"]);

    // Ensure sandbox executed or stubbed gracefully
    assert!(res.is_ok(), "Sandbox execution failed: {:?}", res.err());
}