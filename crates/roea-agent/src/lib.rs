//! roea-agent library
//!
//! Provides the core components for the roea-ai monitoring daemon.
//!
//! On Linux with eBPF support (kernel 5.8+, CAP_BPF/root, BTF), uses kernel
//! tracepoints for real-time process monitoring. Falls back to sysinfo-based
//! polling on other platforms or when eBPF is unavailable.

pub mod file;
pub mod grpc;
pub mod monitor;
pub mod network;
pub mod observability;
pub mod osquery;
pub mod storage;
pub mod telemetry;

pub use file::FileMonitorService;
pub use grpc::{AgentState, RoeaAgentService};
pub use monitor::ProcessMonitorService;
pub use network::NetworkMonitorService;
pub use observability::{init_sentry, metrics, SentryConfig, SentryGuard};
pub use osquery::{OsqueryConfig, OsqueryService};
pub use storage::{Storage, StorageConfig};
pub use telemetry::{TelemetryConfig, TelemetryService};

// Re-export eBPF types when compiled with eBPF support
#[cfg(all(target_os = "linux", ebpf_available))]
pub use monitor::{EbpfProcessMonitor, EbpfError};
