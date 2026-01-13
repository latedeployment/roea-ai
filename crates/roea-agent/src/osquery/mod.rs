//! osquery Integration Module
//!
//! Provides integration with osquery for enhanced system telemetry.
//! Falls back gracefully when osquery is not available.

use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use roea_common::events::{ConnectionInfo, ConnectionState, FileOpInfo, FileOperation, ProcessInfo, Protocol};
use uuid::Uuid;

/// osquery errors
#[derive(Error, Debug)]
pub enum OsqueryError {
    #[error("osquery not available on this system")]
    NotAvailable,
    #[error("osquery query failed: {0}")]
    QueryFailed(String),
    #[error("Failed to parse osquery output: {0}")]
    ParseError(String),
    #[error("osquery socket connection failed: {0}")]
    ConnectionError(String),
}

/// Configuration for osquery integration
#[derive(Debug, Clone)]
pub struct OsqueryConfig {
    /// Path to osqueryi binary (if using CLI mode)
    pub osqueryi_path: Option<String>,
    /// Path to osquery socket (if using socket mode)
    pub socket_path: Option<String>,
    /// Poll interval for scheduled queries
    pub poll_interval: Duration,
    /// Enable diff-based change detection
    pub diff_mode: bool,
}

impl Default for OsqueryConfig {
    fn default() -> Self {
        Self {
            osqueryi_path: None,
            socket_path: None,
            poll_interval: Duration::from_secs(5),
            diff_mode: true,
        }
    }
}

/// osquery integration service
pub struct OsqueryService {
    config: OsqueryConfig,
    available: bool,
    osqueryi_path: String,
    last_processes: Arc<RwLock<HashMap<u32, ProcessInfo>>>,
    last_connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
}

impl OsqueryService {
    /// Create a new osquery service
    pub fn new(config: OsqueryConfig) -> Self {
        let osqueryi_path = config
            .osqueryi_path
            .clone()
            .unwrap_or_else(|| find_osqueryi().unwrap_or_default());

        let available = !osqueryi_path.is_empty() && check_osquery_available(&osqueryi_path);

        if available {
            info!("osquery available at: {}", osqueryi_path);
        } else {
            warn!("osquery not available - using fallback monitoring");
        }

        Self {
            config,
            available,
            osqueryi_path,
            last_processes: Arc::new(RwLock::new(HashMap::new())),
            last_connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if osquery is available
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Query processes using osquery
    pub fn query_processes(&self) -> Result<Vec<ProcessInfo>, OsqueryError> {
        if !self.available {
            return Err(OsqueryError::NotAvailable);
        }

        let query = r#"
            SELECT
                pid,
                parent AS ppid,
                name,
                cmdline,
                path,
                uid,
                cwd,
                start_time
            FROM processes
            WHERE pid > 1
        "#;

        let output = self.execute_query(query)?;
        let rows: Vec<OsqueryProcessRow> = serde_json::from_str(&output)
            .map_err(|e| OsqueryError::ParseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .filter_map(|row| row.try_into().ok())
            .collect())
    }

    /// Query network connections using osquery
    pub fn query_connections(&self) -> Result<Vec<ConnectionInfo>, OsqueryError> {
        if !self.available {
            return Err(OsqueryError::NotAvailable);
        }

        let query = r#"
            SELECT
                pid,
                family,
                protocol,
                local_address,
                local_port,
                remote_address,
                remote_port,
                state
            FROM process_open_sockets
            WHERE pid > 1 AND remote_address != ''
        "#;

        let output = self.execute_query(query)?;
        let rows: Vec<OsquerySocketRow> = serde_json::from_str(&output)
            .map_err(|e| OsqueryError::ParseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .filter_map(|row| row.try_into().ok())
            .collect())
    }

    /// Query open files using osquery
    pub fn query_open_files(&self) -> Result<Vec<FileOpInfo>, OsqueryError> {
        if !self.available {
            return Err(OsqueryError::NotAvailable);
        }

        let query = r#"
            SELECT
                pid,
                fd,
                path
            FROM process_open_files
            WHERE pid > 1 AND path != '' AND path NOT LIKE '/dev/%' AND path NOT LIKE '/proc/%'
        "#;

        let output = self.execute_query(query)?;
        let rows: Vec<OsqueryFileRow> = serde_json::from_str(&output)
            .map_err(|e| OsqueryError::ParseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .filter_map(|row| row.try_into().ok())
            .collect())
    }

    /// Query AI agent processes with enhanced context
    pub fn query_agent_processes(&self, agent_patterns: &[&str]) -> Result<Vec<ProcessInfo>, OsqueryError> {
        if !self.available {
            return Err(OsqueryError::NotAvailable);
        }

        // Build pattern matching WHERE clause
        let patterns: String = agent_patterns
            .iter()
            .map(|p| format!("name LIKE '%{}%' OR cmdline LIKE '%{}%'", p, p))
            .collect::<Vec<_>>()
            .join(" OR ");

        let query = format!(
            r#"
            SELECT
                pid,
                parent AS ppid,
                name,
                cmdline,
                path,
                uid,
                cwd,
                start_time
            FROM processes
            WHERE pid > 1 AND ({})
        "#,
            patterns
        );

        let output = self.execute_query(&query)?;
        let rows: Vec<OsqueryProcessRow> = serde_json::from_str(&output)
            .map_err(|e| OsqueryError::ParseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .filter_map(|row| row.try_into().ok())
            .collect())
    }

    /// Get process with its child processes
    pub fn query_process_tree(&self, root_pid: u32) -> Result<Vec<ProcessInfo>, OsqueryError> {
        if !self.available {
            return Err(OsqueryError::NotAvailable);
        }

        // Recursive CTE to get process tree
        let query = format!(
            r#"
            WITH RECURSIVE process_tree AS (
                SELECT pid, parent, name, cmdline, path, uid, cwd, start_time
                FROM processes
                WHERE pid = {}
                UNION ALL
                SELECT p.pid, p.parent, p.name, p.cmdline, p.path, p.uid, p.cwd, p.start_time
                FROM processes p
                JOIN process_tree pt ON p.parent = pt.pid
            )
            SELECT pid, parent AS ppid, name, cmdline, path, uid, cwd, start_time
            FROM process_tree
        "#,
            root_pid
        );

        let output = self.execute_query(&query)?;
        let rows: Vec<OsqueryProcessRow> = serde_json::from_str(&output)
            .map_err(|e| OsqueryError::ParseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .filter_map(|row| row.try_into().ok())
            .collect())
    }

    /// Get connections for a specific process
    pub fn query_process_connections(&self, pid: u32) -> Result<Vec<ConnectionInfo>, OsqueryError> {
        if !self.available {
            return Err(OsqueryError::NotAvailable);
        }

        let query = format!(
            r#"
            SELECT
                pid,
                family,
                protocol,
                local_address,
                local_port,
                remote_address,
                remote_port,
                state
            FROM process_open_sockets
            WHERE pid = {}
        "#,
            pid
        );

        let output = self.execute_query(&query)?;
        let rows: Vec<OsquerySocketRow> = serde_json::from_str(&output)
            .map_err(|e| OsqueryError::ParseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .filter_map(|row| row.try_into().ok())
            .collect())
    }

    /// Execute a raw SQL query against osquery
    fn execute_query(&self, query: &str) -> Result<String, OsqueryError> {
        let output = Command::new(&self.osqueryi_path)
            .args(["--json", query])
            .output()
            .map_err(|e| OsqueryError::QueryFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(OsqueryError::QueryFailed(stderr.to_string()));
        }

        String::from_utf8(output.stdout).map_err(|e| OsqueryError::ParseError(e.to_string()))
    }

    /// Poll for process changes (diff mode)
    pub fn poll_process_changes(&self) -> Result<ProcessChanges, OsqueryError> {
        let current = self.query_processes()?;
        let mut last = self.last_processes.write();

        let current_map: HashMap<u32, ProcessInfo> = current
            .iter()
            .map(|p| (p.pid, p.clone()))
            .collect();

        let mut spawned = Vec::new();
        let mut exited = Vec::new();

        // Find new processes
        for (pid, process) in &current_map {
            if !last.contains_key(pid) {
                spawned.push(process.clone());
            }
        }

        // Find exited processes
        for (pid, process) in last.iter() {
            if !current_map.contains_key(pid) {
                exited.push(process.clone());
            }
        }

        *last = current_map;

        Ok(ProcessChanges { spawned, exited })
    }
}

/// Process changes detected by diff mode
pub struct ProcessChanges {
    pub spawned: Vec<ProcessInfo>,
    pub exited: Vec<ProcessInfo>,
}

// osquery JSON row types

#[derive(Debug, Deserialize)]
struct OsqueryProcessRow {
    pid: String,
    ppid: String,
    name: String,
    cmdline: Option<String>,
    path: Option<String>,
    uid: Option<String>,
    cwd: Option<String>,
    start_time: Option<String>,
}

impl TryFrom<OsqueryProcessRow> for ProcessInfo {
    type Error = OsqueryError;

    fn try_from(row: OsqueryProcessRow) -> Result<Self, Self::Error> {
        let pid: u32 = row
            .pid
            .parse()
            .map_err(|_| OsqueryError::ParseError("Invalid PID".to_string()))?;
        let ppid: u32 = row
            .ppid
            .parse()
            .map_err(|_| OsqueryError::ParseError("Invalid PPID".to_string()))?;

        let start_time = row
            .start_time
            .and_then(|s| s.parse::<i64>().ok())
            .map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_default())
            .unwrap_or_else(Utc::now);

        Ok(ProcessInfo {
            id: Uuid::new_v4(),
            pid,
            ppid: Some(ppid),
            name: row.name,
            cmdline: row.cmdline,
            exe_path: row.path,
            user: row.uid,
            cwd: row.cwd,
            start_time,
            end_time: None,
            agent_type: None,
        })
    }
}

#[derive(Debug, Deserialize)]
struct OsquerySocketRow {
    pid: String,
    family: Option<String>,
    protocol: Option<String>,
    local_address: Option<String>,
    local_port: Option<String>,
    remote_address: Option<String>,
    remote_port: Option<String>,
    state: Option<String>,
}

impl TryFrom<OsquerySocketRow> for ConnectionInfo {
    type Error = OsqueryError;

    fn try_from(row: OsquerySocketRow) -> Result<Self, Self::Error> {
        let pid: u32 = row
            .pid
            .parse()
            .map_err(|_| OsqueryError::ParseError("Invalid PID".to_string()))?;

        let protocol = match row.protocol.as_deref() {
            Some("6") | Some("tcp") => Protocol::Tcp,
            Some("17") | Some("udp") => Protocol::Udp,
            _ => Protocol::Tcp,
        };

        let state = match row.state.as_deref() {
            Some("ESTABLISHED") | Some("01") => ConnectionState::Established,
            Some("LISTEN") | Some("0A") => ConnectionState::Listen,
            Some("TIME_WAIT") | Some("06") => ConnectionState::TimeWait,
            Some("CLOSE_WAIT") | Some("08") => ConnectionState::CloseWait,
            _ => ConnectionState::Unknown,
        };

        Ok(ConnectionInfo {
            id: Uuid::new_v4(),
            process_id: None,
            pid,
            protocol,
            local_addr: row.local_address,
            local_port: row.local_port.and_then(|p| p.parse().ok()),
            remote_addr: row.remote_address,
            remote_port: row.remote_port.and_then(|p| p.parse().ok()),
            state,
            timestamp: Utc::now(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct OsqueryFileRow {
    pid: String,
    fd: Option<String>,
    path: String,
}

impl TryFrom<OsqueryFileRow> for FileOpInfo {
    type Error = OsqueryError;

    fn try_from(row: OsqueryFileRow) -> Result<Self, Self::Error> {
        let pid: u32 = row
            .pid
            .parse()
            .map_err(|_| OsqueryError::ParseError("Invalid PID".to_string()))?;

        Ok(FileOpInfo {
            id: Uuid::new_v4(),
            process_id: None,
            pid,
            operation: FileOperation::Open,
            path: row.path,
            new_path: None,
            timestamp: Utc::now(),
        })
    }
}

/// Find osqueryi binary on the system
fn find_osqueryi() -> Option<String> {
    let paths = [
        "/usr/bin/osqueryi",
        "/usr/local/bin/osqueryi",
        "/opt/osquery/bin/osqueryi",
        "osqueryi", // Use PATH
    ];

    for path in paths {
        if check_osquery_available(path) {
            return Some(path.to_string());
        }
    }

    // Try using 'which' on Unix
    #[cfg(unix)]
    {
        if let Ok(output) = Command::new("which").arg("osqueryi").output() {
            if output.status.success() {
                if let Ok(path) = String::from_utf8(output.stdout) {
                    let path = path.trim().to_string();
                    if !path.is_empty() {
                        return Some(path);
                    }
                }
            }
        }
    }

    None
}

/// Check if osquery is available at the given path
fn check_osquery_available(path: &str) -> bool {
    Command::new(path)
        .args(["--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osquery_service_creation() {
        let config = OsqueryConfig::default();
        let service = OsqueryService::new(config);
        // Service should be created regardless of osquery availability
        // If osquery is not installed, is_available() returns false
        let _ = service.is_available();
    }

    #[test]
    fn test_process_row_conversion() {
        let row = OsqueryProcessRow {
            pid: "1234".to_string(),
            ppid: "1".to_string(),
            name: "test_process".to_string(),
            cmdline: Some("/usr/bin/test --flag".to_string()),
            path: Some("/usr/bin/test".to_string()),
            uid: Some("1000".to_string()),
            cwd: Some("/home/user".to_string()),
            start_time: Some("1704067200".to_string()),
        };

        let process: Result<ProcessInfo, _> = row.try_into();
        assert!(process.is_ok());

        let process = process.unwrap();
        assert_eq!(process.pid, 1234);
        assert_eq!(process.name, "test_process");
    }
}
