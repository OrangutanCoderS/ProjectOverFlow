use super::ThrottleController;

pub struct MockThrottle;

impl ThrottleController for MockThrottle {
    fn limit_cpu(pid: i32, intensity: f32) -> Result<(), String> {
        println!("[mock] limit_cpu pid={pid} intensity={intensity}");
        Ok(())
    }

    fn limit_io(pid: i32, intensity: f32) -> Result<(), String> {
        println!("[mock] limit_io pid={pid} intensity={intensity}");
        Ok(())
    }

    fn limit_net(pid: i32, intensity: f32) -> Result<(), String> {
        println!("[mock] limit_net pid={pid} intensity={intensity}");
        Ok(())
    }

    fn restore(pid: i32) -> Result<(), String> {
        println!("[mock] restore pid={pid}");
        Ok(())
    }
}