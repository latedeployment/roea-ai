//! Process monitoring implementations
//!
//! Provides platform-specific and fallback process monitoring.
//!
//! On Linux with eBPF support, uses kernel tracepoints for real-time
//! process monitoring. Falls back to sysinfo polling on other platforms
//! or when eBPF is not available.
//!
//! To enable eBPF on Linux:
//! 1. Generate vmlinux.h: `bpftool btf dump file /sys/kernel/btf/vmlinux format c > src/bpf/vmlinux.h`
//! 2. Rebuild the crate
//! 3. Run as root or with CAP_BPF capability

mod sysinfo_monitor;

#[cfg(test)]
mod tests;

#[cfg(all(target_os = "linux", ebpf_available))]
mod ebpf_monitor;

pub use sysinfo_monitor::SysinfoMonitor;

#[cfg(all(target_os = "linux", ebpf_available))]
pub use ebpf_monitor::{EbpfProcessMonitor, EbpfError};

use roea_common::{ProcessEvent, ProcessInfo, PlatformError, PlatformResult};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Unified process monitor that can use different backends
pub struct ProcessMonitorService {
    inner: Box<dyn ProcessMonitorBackend>,
    event_tx: broadcast::Sender<ProcessEvent>,
    #[allow(dead_code)]
    backend_name: &'static str,
}

impl ProcessMonitorService {
    /// Create a new process monitor service
    ///
    /// On Linux with eBPF compiled in, attempts to use kernel tracepoints
    /// for real-time process monitoring. Falls back to sysinfo polling
    /// if eBPF is not available at runtime.
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);

        // Try eBPF on Linux first (only when compiled with eBPF support)
        #[cfg(all(target_os = "linux", ebpf_available))]
        {
            if EbpfProcessMonitor::is_available() {
                info!("eBPF process monitoring available, using kernel tracepoints");
                return Self {
                    inner: Box::new(EbpfProcessMonitor::new()),
                    event_tx,
                    backend_name: "ebpf",
                };
            } else {
                warn!("eBPF not available at runtime (requires root/CAP_BPF and BTF support), falling back to sysinfo");
            }
        }

        #[cfg(all(target_os = "linux", not(ebpf_available)))]
        {
            info!("eBPF not compiled in (vmlinux.h not found during build)");
        }

        // Fallback to sysinfo-based monitoring
        info!("Using sysinfo-based process monitoring");
        Self {
            inner: Box::new(SysinfoMonitor::new()),
            event_tx,
            backend_name: "sysinfo",
        }
    }

    /// Get the name of the active backend
    pub fn backend_name(&self) -> &'static str {
        self.backend_name
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
