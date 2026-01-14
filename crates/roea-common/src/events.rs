//! Event types for process, network, and file monitoring

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for processes tracked by roea-ai
pub type ProcessId = Uuid;

/// Event type for process lifecycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessEventType {
    /// Process was spawned
    Spawn,
    /// Process exited
    Exit,
    /// Process metadata updated
    Update,
}

/// Process information captured during monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Internal unique identifier
    pub id: ProcessId,
    /// Operating system process ID
    pub pid: u32,
    /// Parent process ID
    pub ppid: Option<u32>,
    /// Process name (executable name)
    pub name: String,
    /// Full command line
    pub cmdline: Option<String>,
    /// Path to executable
    pub exe_path: Option<String>,
    /// Detected agent type (e.g., "claude_code", "cursor")
    pub agent_type: Option<String>,
    /// Process start time
    pub start_time: DateTime<Utc>,
    /// Process end time (if exited)
    pub end_time: Option<DateTime<Utc>>,
    /// User running the process
    pub user: Option<String>,
    /// Working directory
    pub cwd: Option<String>,
}

impl ProcessInfo {
    /// Create a new ProcessInfo with a generated UUID
    pub fn new(pid: u32, name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            pid,
            ppid: None,
            name,
            cmdline: None,
            exe_path: None,
            agent_type: None,
            start_time: Utc::now(),
            end_time: None,
            user: None,
            cwd: None,
        }
    }
}

/// A process event with type and process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvent {
    pub event_type: ProcessEventType,
    pub process: ProcessInfo,
    pub timestamp: DateTime<Utc>,
}

/// Network protocol type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
    Unix,
}

/// Network connection state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    Connecting,
    Established,
    Listen,
    TimeWait,
    CloseWait,
    Closed,
    Unknown,
}

/// Network connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Internal unique identifier
    pub id: Uuid,
    /// Associated process ID (internal)
    pub process_id: Option<ProcessId>,
    /// Operating system PID
    pub pid: u32,
    /// Protocol type
    pub protocol: Protocol,
    /// Local address
    pub local_addr: Option<String>,
    /// Local port
    pub local_port: Option<u16>,
    /// Remote address
    pub remote_addr: Option<String>,
    /// Remote port
    pub remote_port: Option<u16>,
    /// Connection state
    pub state: ConnectionState,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
}

impl ConnectionInfo {
    /// Create a new ConnectionInfo with a generated UUID
    pub fn new(pid: u32, protocol: Protocol) -> Self {
        Self {
            id: Uuid::new_v4(),
            process_id: None,
            pid,
            protocol,
            local_addr: None,
            local_port: None,
            remote_addr: None,
            remote_port: None,
            state: ConnectionState::Connecting,
            timestamp: Utc::now(),
        }
    }
}

/// File operation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileOperation {
    Open,
    Read,
    Write,
    Delete,
    Rename,
    Create,
}

/// File operation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOpInfo {
    /// Internal unique identifier
    pub id: Uuid,
    /// Associated process ID (internal)
    pub process_id: Option<ProcessId>,
    /// Operating system PID
    pub pid: u32,
    /// Type of operation
    pub operation: FileOperation,
    /// File path
    pub path: String,
    /// New path (for rename operations)
    pub new_path: Option<String>,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
}

impl FileOpInfo {
    /// Create a new FileOpInfo with a generated UUID
    pub fn new(pid: u32, operation: FileOperation, path: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            process_id: None,
            pid,
            operation,
            path,
            new_path: None,
            timestamp: Utc::now(),
        }
    }
}

/// Unified telemetry event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TelemetryEvent {
    Process(ProcessEvent),
    Connection(ConnectionInfo),
    FileOp(FileOpInfo),
}
