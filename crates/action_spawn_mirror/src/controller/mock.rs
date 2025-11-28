use super::MirrorController;

pub struct MockMirror;
impl MirrorController for MockMirror {
    fn snapshot(_pid: i32) -> Result<(String, Vec<String>), String> {
        Ok(("/usr/bin/env".to_string(), vec!["env".into()]))
    }
    fn spawn_clone(_cmd: &str, _args: &[String]) -> Result<u32, String> {
        Ok(99999)
    }
}