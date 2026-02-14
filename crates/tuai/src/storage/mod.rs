//! DuckDB-based storage layer for telemetry data
//!
//! Provides persistent storage for process, network, and file events
//! with time-series optimized queries.

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use duckdb::{params, Connection};
use parking_lot::Mutex;
use tuai_common::{
    ConnectionInfo, ConnectionState, FileOpInfo, ProcessInfo, Protocol,
};
use uuid::Uuid;

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Database file path (None for in-memory)
    pub db_path: Option<String>,
    /// Data retention period in hours
    pub retention_hours: u32,
    /// Whether to run cleanup on startup
    pub cleanup_on_start: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            db_path: None,
            retention_hours: 24,
            cleanup_on_start: true,
        }
    }
}

/// Storage layer wrapping DuckDB
pub struct Storage {
    conn: Arc<Mutex<Connection>>,
    config: StorageConfig,
}

impl Storage {
    /// Create a new storage instance
    pub fn new(config: StorageConfig) -> Result<Self> {
        let conn = if let Some(ref path) = config.db_path {
            // Ensure parent directory exists
            if let Some(parent) = Path::new(path).parent() {
                std::fs::create_dir_all(parent)
                    .context("Failed to create database directory")?;
            }
            Connection::open(path)?
        } else {
            Connection::open_in_memory()?
        };

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
            config,
        };

        storage.initialize_schema()?;

        if storage.config.cleanup_on_start {
            storage.cleanup_old_data()?;
        }

        Ok(storage)
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.lock();

        // Process events table
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS processes (
                id VARCHAR PRIMARY KEY,
                pid INTEGER NOT NULL,
                ppid INTEGER,
                name VARCHAR NOT NULL,
                cmdline VARCHAR,
                exe_path VARCHAR,
                agent_type VARCHAR,
                start_time TIMESTAMP NOT NULL,
                end_time TIMESTAMP,
                user_name VARCHAR,
                cwd VARCHAR
            );

            CREATE INDEX IF NOT EXISTS idx_processes_agent ON processes(agent_type);
            CREATE INDEX IF NOT EXISTS idx_processes_time ON processes(start_time);
            CREATE INDEX IF NOT EXISTS idx_processes_pid ON processes(pid);
            "#,
        )?;

        // Network connections table
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS connections (
                id VARCHAR PRIMARY KEY,
                process_id VARCHAR,
                pid INTEGER NOT NULL,
                protocol VARCHAR,
                local_addr VARCHAR,
                local_port INTEGER,
                remote_addr VARCHAR,
                remote_port INTEGER,
                state VARCHAR,
                timestamp TIMESTAMP NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_connections_remote ON connections(remote_addr);
            CREATE INDEX IF NOT EXISTS idx_connections_time ON connections(timestamp);
            CREATE INDEX IF NOT EXISTS idx_connections_pid ON connections(pid);
            "#,
        )?;

        // File operations table
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS file_ops (
                id VARCHAR PRIMARY KEY,
                process_id VARCHAR,
                pid INTEGER NOT NULL,
                operation VARCHAR,
                path VARCHAR NOT NULL,
                new_path VARCHAR,
                timestamp TIMESTAMP NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_file_ops_path ON file_ops(path);
            CREATE INDEX IF NOT EXISTS idx_file_ops_time ON file_ops(timestamp);
            CREATE INDEX IF NOT EXISTS idx_file_ops_pid ON file_ops(pid);
            "#,
        )?;

        Ok(())
    }

    /// Insert a process event
    pub fn insert_process(&self, process: &ProcessInfo) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            r#"
            INSERT INTO processes (id, pid, ppid, name, cmdline, exe_path, agent_type, start_time, end_time, user_name, cwd)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT (id) DO UPDATE SET
                end_time = excluded.end_time,
                agent_type = COALESCE(excluded.agent_type, processes.agent_type)
            "#,
            params![
                process.id.to_string(),
                process.pid,
                process.ppid,
                process.name,
                process.cmdline,
                process.exe_path,
                process.agent_type,
                process.start_time.to_rfc3339(),
                process.end_time.map(|t| t.to_rfc3339()),
                process.user,
                process.cwd,
            ],
        )?;
        Ok(())
    }

    /// Update process end time
    pub fn update_process_exit(&self, id: &Uuid, end_time: DateTime<Utc>) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE processes SET end_time = ? WHERE id = ?",
            params![end_time.to_rfc3339(), id.to_string()],
        )?;
        Ok(())
    }

    /// Insert a network connection event
    pub fn insert_connection(&self, conn_info: &ConnectionInfo) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            r#"
            INSERT INTO connections (id, process_id, pid, protocol, local_addr, local_port, remote_addr, remote_port, state, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                conn_info.id.to_string(),
                conn_info.process_id.map(|id| id.to_string()),
                conn_info.pid,
                format!("{:?}", conn_info.protocol).to_lowercase(),
                conn_info.local_addr,
                conn_info.local_port,
                conn_info.remote_addr,
                conn_info.remote_port,
                format!("{:?}", conn_info.state).to_lowercase(),
                conn_info.timestamp.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Insert a file operation event
    pub fn insert_file_op(&self, file_op: &FileOpInfo) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            r#"
            INSERT INTO file_ops (id, process_id, pid, operation, path, new_path, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                file_op.id.to_string(),
                file_op.process_id.map(|id| id.to_string()),
                file_op.pid,
                format!("{:?}", file_op.operation).to_lowercase(),
                file_op.path,
                file_op.new_path,
                file_op.timestamp.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Query processes within a time range
    pub fn query_processes(
        &self,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        agent_type: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ProcessInfo>> {
        let conn = self.conn.lock();

        let mut sql = String::from("SELECT id, pid, ppid, name, cmdline, exe_path, agent_type, start_time, end_time, user_name, cwd FROM processes WHERE 1=1");

        if start_time.is_some() {
            sql.push_str(" AND start_time >= ?");
        }
        if end_time.is_some() {
            sql.push_str(" AND start_time <= ?");
        }
        if agent_type.is_some() {
            sql.push_str(" AND agent_type = ?");
        }

        sql.push_str(" ORDER BY start_time DESC LIMIT ? OFFSET ?");

        let mut stmt = conn.prepare(&sql)?;

        // Build params dynamically
        let mut params_vec: Vec<Box<dyn duckdb::ToSql>> = Vec::new();

        if let Some(st) = start_time {
            params_vec.push(Box::new(st.to_rfc3339()));
        }
        if let Some(et) = end_time {
            params_vec.push(Box::new(et.to_rfc3339()));
        }
        if let Some(at) = agent_type {
            params_vec.push(Box::new(at.to_string()));
        }
        params_vec.push(Box::new(limit as i64));
        params_vec.push(Box::new(offset as i64));

        let params_refs: Vec<&dyn duckdb::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(ProcessInfo {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_else(|_| Uuid::new_v4()),
                pid: row.get::<_, i32>(1)? as u32,
                ppid: row.get::<_, Option<i32>>(2)?.map(|p| p as u32),
                name: row.get(3)?,
                cmdline: row.get(4)?,
                exe_path: row.get(5)?,
                agent_type: row.get(6)?,
                start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                end_time: row.get::<_, Option<String>>(8)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                user: row.get(9)?,
                cwd: row.get(10)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Query connections for a process
    pub fn query_connections_by_pid(&self, pid: u32, limit: usize) -> Result<Vec<ConnectionInfo>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, process_id, pid, protocol, local_addr, local_port, remote_addr, remote_port, state, timestamp
             FROM connections WHERE pid = ? ORDER BY timestamp DESC LIMIT ?"
        )?;

        let rows = stmt.query_map(params![pid as i32, limit as i64], |row| {
            Ok(ConnectionInfo {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_else(|_| Uuid::new_v4()),
                process_id: row.get::<_, Option<String>>(1)?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                pid: row.get::<_, i32>(2)? as u32,
                protocol: match row.get::<_, String>(3)?.as_str() {
                    "udp" => Protocol::Udp,
                    "unix" => Protocol::Unix,
                    _ => Protocol::Tcp,
                },
                local_addr: row.get(4)?,
                local_port: row.get::<_, Option<i32>>(5)?.map(|p| p as u16),
                remote_addr: row.get(6)?,
                remote_port: row.get::<_, Option<i32>>(7)?.map(|p| p as u16),
                state: match row.get::<_, String>(8)?.as_str() {
                    "established" => ConnectionState::Established,
                    "closed" => ConnectionState::Closed,
                    _ => ConnectionState::Connecting,
                },
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get process count
    pub fn process_count(&self) -> Result<i64> {
        let conn = self.conn.lock();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM processes", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Get total event count (all tables)
    pub fn total_event_count(&self) -> Result<i64> {
        let conn = self.conn.lock();
        let processes: i64 = conn.query_row("SELECT COUNT(*) FROM processes", [], |row| row.get(0))?;
        let connections: i64 = conn.query_row("SELECT COUNT(*) FROM connections", [], |row| row.get(0))?;
        let file_ops: i64 = conn.query_row("SELECT COUNT(*) FROM file_ops", [], |row| row.get(0))?;
        Ok(processes + connections + file_ops)
    }

    /// Cleanup data older than retention period
    pub fn cleanup_old_data(&self) -> Result<usize> {
        let conn = self.conn.lock();
        let cutoff = Utc::now() - chrono::Duration::hours(self.config.retention_hours as i64);
        let cutoff_str = cutoff.to_rfc3339();

        let mut deleted = 0;

        deleted += conn.execute(
            "DELETE FROM file_ops WHERE timestamp < ?",
            params![&cutoff_str],
        )?;

        deleted += conn.execute(
            "DELETE FROM connections WHERE timestamp < ?",
            params![&cutoff_str],
        )?;

        deleted += conn.execute(
            "DELETE FROM processes WHERE start_time < ? AND end_time IS NOT NULL",
            params![&cutoff_str],
        )?;

        Ok(deleted)
    }
}

// Need to add parking_lot to dependencies
