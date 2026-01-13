//! roea-agent library
//!
//! Provides the core components for the roea-ai monitoring daemon.

pub mod file;
pub mod grpc;
pub mod monitor;
pub mod network;
pub mod storage;
pub mod telemetry;

pub use file::FileMonitorService;
pub use grpc::{AgentState, RoeaAgentService};
pub use monitor::ProcessMonitorService;
pub use network::NetworkMonitorService;
pub use storage::{Storage, StorageConfig};
pub use telemetry::{TelemetryConfig, TelemetryService};
