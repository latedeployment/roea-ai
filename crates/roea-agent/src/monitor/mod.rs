//! Process monitoring implementations
//!
//! Provides platform-specific and fallback process monitoring.

mod sysinfo_monitor;

pub use sysinfo_monitor::SysinfoMonitor;

use roea_common::{ProcessEvent, ProcessInfo, PlatformError, PlatformResult};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Unified process monitor that can use different backends
pub struct ProcessMonitorService {
    inner: Box<dyn ProcessMonitorBackend>,
    event_tx: broadcast::Sender<ProcessEvent>,
}

impl ProcessMonitorService {
    /// Create a new process monitor service
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        Self {
            inner: Box::new(SysinfoMonitor::new()),
            event_tx,
        }
    }

    /// Start monitoring
    pub fn start(&mut self) -> PlatformResult<()> {
        self.inner.start()
    }

    /// Stop monitoring
    pub fn stop(&mut self) -> PlatformResult<()> {
        self.inner.stop()
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    /// Get current process snapshot
    pub fn snapshot(&self) -> PlatformResult<Vec<ProcessInfo>> {
        self.inner.snapshot()
    }

    /// Subscribe to process events
    pub fn subscribe(&self) -> broadcast::Receiver<ProcessEvent> {
        self.event_tx.subscribe()
    }

    /// Get the event sender for publishing events
    pub fn event_sender(&self) -> broadcast::Sender<ProcessEvent> {
        self.event_tx.clone()
    }
}

impl Default for ProcessMonitorService {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for process monitoring backends
pub trait ProcessMonitorBackend: Send + Sync {
    fn start(&mut self) -> PlatformResult<()>;
    fn stop(&mut self) -> PlatformResult<()>;
    fn is_running(&self) -> bool;
    fn snapshot(&self) -> PlatformResult<Vec<ProcessInfo>>;
}
