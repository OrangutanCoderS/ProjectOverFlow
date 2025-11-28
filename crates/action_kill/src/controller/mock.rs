use std::sync::{Arc, RwLock};
use crate::errors::ManagerError;
use super::ProcessController;

#[derive(Default, Clone)]
pub struct MockProcController {
    pids: Arc<RwLock<Vec<i32>>>,
}

impl MockProcController {
    pub fn new() -> Self { Self::default() }
    pub fn insert(&self, pid: i32) { self.pids.write().unwrap().push(pid); }
}

impl ProcessController for MockProcController {
    fn exists(&self, pid: i32) -> bool {
        self.pids.read().unwrap().contains(&pid)
    }

    fn send_signal(&self, pid: i32, _signal: i32) -> Result<(), ManagerError> {
        let mut table = self.pids.write().unwrap();
        if let Some(pos) = table.iter().position(|&p| p == pid) {
            table.remove(pos);
            Ok(())
        } else {
            Err(ManagerError::NotFound(pid))
        }
    }
}