use super::ThrottleController;
use nix::sys::resource::{setpriority, PriorityWhich};
use std::thread;
use std::time::Duration;

pub struct LinuxThrottle;

impl ThrottleController for LinuxThrottle {
    fn limit_cpu(pid: i32, intensity: f32) -> Result<(), String> {
        let nice_val = ((1.0 - intensity.clamp(0.0, 1.0)) * 19.0).round() as i32;
        setpriority(PriorityWhich::PRIO_PROCESS, pid as u32, nice_val)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn limit_io(pid: i32, intensity: f32) -> Result<(), String> {
        if intensity >= 0.8 {
            thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    }

    fn limit_net(pid: i32, intensity: f32) -> Result<(), String> {
        println!(
            "[simulated] Network throttle pid={pid} intensity={intensity:.2}"
        );
        Ok(())
    }

    fn restore(pid: i32) -> Result<(), String> {
        setpriority(PriorityWhich::PRIO_PROCESS, pid as u32, 0)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}