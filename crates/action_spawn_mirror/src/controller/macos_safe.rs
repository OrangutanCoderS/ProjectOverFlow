use super::MirrorController;
use psutil::process::Process;

pub struct MacOSMirror;

impl MirrorController for MacOSMirror {
    fn snapshot(pid: i32) -> Result<(String, Vec<String>), String> {
        // psutil requires u32 PID
        let proc = Process::new(pid as u32).map_err(|e| e.to_string())?;

        // cmdline() on macOS returns Option<String> (space-separated)
        let cmd_opt = proc.cmdline().map_err(|e| e.to_string())?;
        let cmdline_raw = cmd_opt.ok_or("cmdline unavailable")?;

        // Split the raw string into arguments safely
        let cmdline_vec: Vec<String> = cmdline_raw
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        // Defensive fallback
        if cmdline_vec.is_empty() {
            return Err("empty cmdline".into());
        }

        let program = cmdline_vec[0].clone();
        Ok((program, cmdline_vec))
    }

    fn spawn_clone(cmd: &str, args: &[String]) -> Result<u32, String> {
        std::process::Command::new(cmd)
            .args(args.iter().skip(1))
            .spawn()
            .map(|child| child.id())
            .map_err(|e| e.to_string())
    }
}