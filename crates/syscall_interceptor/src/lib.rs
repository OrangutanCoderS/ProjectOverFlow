//! Syscall Interceptor — Phase II skeleton
//!
//! Provides a stable Rust API for observing syscalls.
//! Currently backed by mock events (safe, no root).
//! Later phases can plug in C shims (ptrace/seccomp, dtrace/ktrace).

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]

pub mod error;
use error::InterceptorError;

use crossbeam_channel::{unbounded, Receiver, Sender};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use tracing::info;

/// Represents a syscall observation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyscallEvent {
    pub pid: i32,
    pub name: String,
    pub args: Vec<String>,
    pub timestamp: String,
}

/// Global interceptor handle (singleton-ish).
pub struct SyscallInterceptor {
    tx: Sender<SyscallEvent>,
    rx: Receiver<SyscallEvent>,
    attached: bool,
}

/// Thread-safe singleton.
pub static GLOBAL_INTERCEPTOR: Lazy<RwLock<Option<SyscallInterceptor>>> =
    Lazy::new(|| RwLock::new(None));

impl SyscallInterceptor {
    /// Create a new interceptor (mock backend).
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self {
            tx,
            rx,
            attached: false,
        }
    }

    /// Attach to system (mock mode: just marks as attached).
    pub fn attach(&mut self) -> Result<(), InterceptorError> {
        if self.attached {
            return Err(InterceptorError::InvalidOperation("already attached".into()));
        }
        info!("syscall_interceptor: attached (mock)");
        self.attached = true;
        Ok(())
    }

    /// Detach from system.
    pub fn detach(&mut self) -> Result<(), InterceptorError> {
        if !self.attached {
            return Err(InterceptorError::InvalidOperation("not attached".into()));
        }
        self.attached = false;
        info!("syscall_interceptor: detached (mock)");
        Ok(())
    }

    /// Push a mock event (for tests/benchmarks).
    pub fn push_mock(&self, ev: SyscallEvent) -> Result<(), InterceptorError> {
        self.tx
            .send(ev)
            .map_err(|_| InterceptorError::ChannelClosed)
    }

    /// Non-blocking receive of a syscall event.
    pub fn try_recv(&self) -> Result<Option<SyscallEvent>, InterceptorError> {
        match self.rx.try_recv() {
            Ok(ev) => Ok(Some(ev)),
            Err(crossbeam_channel::TryRecvError::Empty) => Ok(None),
            Err(_) => Err(InterceptorError::ChannelClosed),
        }
    }
}