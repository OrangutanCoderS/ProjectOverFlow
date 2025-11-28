use action_spawn_mirror::{ActionSpawnMirror, MirrorRequest};

#[test]
fn test_invalid_pid() {
    let mgr = ActionSpawnMirror::default();
    let req = MirrorRequest { pid: -1, sandboxed: false, context: Some("unit_invalid".into()) };
    assert!(mgr.execute(req).is_err());
}

#[test]
fn test_mock_sandbox_spawn() {
    let mgr = ActionSpawnMirror::default();
    let pid = std::process::id() as i32;
    let req = MirrorRequest { pid, sandboxed: false, context: Some("unit_spawn".into()) };
    let res = mgr.execute(req);
    assert!(res.is_ok());
}