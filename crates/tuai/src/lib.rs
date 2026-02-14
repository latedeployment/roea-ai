//! tuai - TUI for monitoring AI coding agents
//!
//! A k9s-like terminal interface for real-time AI agent monitoring.
//!
//! On Linux with eBPF support (kernel 5.8+, CAP_BPF/root, BTF), uses kernel
//! tracepoints for real-time process monitoring. Falls back to sysinfo-based
//! polling on other platforms or when eBPF is unavailable.

#![allow(dead_code)]

pub mod file;
pub mod grpc;
pub mod monitor;
pub mod network;
pub mod protection;
pub mod storage;
pub mod tui;

pub use file::FileMonitorService;
pub use grpc::{AgentState, TuaiAgentService};
pub use monitor::ProcessMonitorService;
pub use network::NetworkMonitorService;
pub use protection::{ProtectionConfig, ProtectionEvent, ProtectionService};
pub use storage::{Storage, StorageConfig};

// Re-export eBPF types when compiled with eBPF support
#[cfg(all(target_os = "linux", ebpf_available))]
pub use monitor::{EbpfProcessMonitor, EbpfError};
