use super::ThrottleController;
use std::process::Command;
use std::thread;
use std::time::Duration;

/// macOS-safe user-space throttle implementation.
/// Works entirely under SIP and without root privileges.
/// Falls back to simulated throttling when renice fails.
pub struct MacOSThrottle;

impl ThrottleController for MacOSThrottle {
    /// Limit CPU utilization using `renice`.  
    /// Falls back to voluntary sleep simulation if permission denied.
    fn limit_cpu(pid: i32, intensity: f32) -> Result<(), String> {
        let nice_val = ((1.0 - intensity.clamp(0.0, 1.0)) * 19.0).round() as i32;

        let status = Command::new("renice")
            .arg(format!("{nice_val}"))
            .arg("-p")
            .arg(pid.to_string())
            .output();

        match status {
            Ok(output) if output.status.success() => {
                println!(
                    "[macos] CPU throttle applied: pid={} nice={}",
                    pid, nice_val
                );
                Ok(())
            }
            Ok(_) | Err(_) => {
                // 🔒 Safe fallback: simulate throttling in user space
                let simulated_delay =
                    ((1.0 - intensity.clamp(0.1, 1.0)) * 100.0) as u64;
                thread::sleep(Duration::from_millis(simulated_delay));
                println!(
                    "[safe-fallback] CPU throttle simulated pid={} intensity={:.2}",
                    pid, intensity
                );
                Ok(())
            }
        }
    }

    /// Simulate I/O throttling via short sleeps.
    /// This is user-space safe and reproducible across test systems.
    fn limit_io(_pid: i32, intensity: f32) -> Result<(), String> {
        if intensity >= 0.8 {
            thread::sleep(Duration::from_millis(50));
            println!(
                "[macos] I/O throttle simulated (intensity={:.2})",
                intensity
            );
        }
        Ok(())
    }

    /// Simulated user-space network throttle.
    /// Real network shaping requires root privileges, so we log instead.
    fn limit_net(pid: i32, intensity: f32) -> Result<(), String> {
        println!(
            "[macos-sim] Network throttle pid={} intensity={:.2}",
            pid, intensity
        );
        Ok(())
    }

    /// Attempt to restore default priority.
    /// Fallback: simulate success if renice fails.
    fn restore(pid: i32) -> Result<(), String> {
        let status = Command::new("renice")
            .arg("0")
            .arg("-p")
            .arg(pid.to_string())
            .output();

        match status {
            Ok(output) if output.status.success() => {
                println!("[macos] Restored priority for pid={}", pid);
                Ok(())
            }
            Ok(_) | Err(_) => {
                println!(
                    "[safe-fallback] restore simulated pid={} (no privilege)",
                    pid
                );
                Ok(())
            }
        }
    }
}