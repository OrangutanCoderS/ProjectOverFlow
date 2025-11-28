use super::MirrorController;
use psutil::process::Process;

pub struct LinuxMirror;

impl MirrorController for LinuxMirror {
    fn snapshot(pid: i32) -> Result<(String, Vec<String>), String> {
        let proc = Process::new(pid).map_err(|e| e.to_string())?;
        let cmdline = proc.cmdline().map_err(|e| e.to_string())?;
        let program = cmdline.get(0).cloned().ok_or("empty cmdline")?;
        Ok((program, cmdline))
    }

    fn spawn_clone(cmd: &str, args: &[String]) -> Result<u32, String> {
        std::process::Command::new(cmd)
            .args(args.iter().skip(1))
            .spawn()
            .map(|child| child.id())
            .map_err(|e| e.to_string())
    }
}