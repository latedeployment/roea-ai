//! Platform abstraction traits for cross-platform monitoring
//!
//! These traits define the interface that each platform-specific implementation
//! must provide for process, network, and file monitoring.

use std::pin::Pin;

use thiserror::Error;

use crate::events::{ConnectionInfo, FileOpInfo, ProcessEvent};

/// Errors that can occur during platform monitoring operations
#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Platform not supported: {0}")]
    NotSupported(String),

    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Event collection failed: {0}")]
    CollectionFailed(String),

    #[error("Resource unavailable: {0}")]
    ResourceUnavailable(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for platform operations
pub type PlatformResult<T> = Result<T, PlatformError>;

/// Async stream of events
pub type EventStream<T> = Pin<Box<dyn futures_core::Stream<Item = PlatformResult<T>> + Send>>;

/// Trait for process monitoring implementations
///
/// Each platform (Linux, macOS, Windows) provides its own implementation
/// using the most efficient available mechanism (eBPF, Endpoint Security, ETW).
pub trait ProcessMonitor: Send + Sync {
    /// Start the process monitor
    ///
    /// This should initialize any required resources (eBPF programs, etc.)
    /// and begin collecting events.
    fn start(&mut self) -> PlatformResult<()>;

    /// Stop the process monitor
    fn stop(&mut self) -> PlatformResult<()>;

    /// Check if the monitor is currently running
    fn is_running(&self) -> bool;

    /// Get a stream of process events
    ///
    /// Returns an async stream that yields process events as they occur.
    fn events(&self) -> EventStream<ProcessEvent>;

    /// Get current running processes (snapshot)
    fn snapshot(&self) -> PlatformResult<Vec<crate::events::ProcessInfo>>;
}

/// Trait for network connection monitoring implementations
pub trait NetworkMonitor: Send + Sync {
    /// Start the network monitor
    fn start(&mut self) -> PlatformResult<()>;

    /// Stop the network monitor
    fn stop(&mut self) -> PlatformResult<()>;

    /// Check if the monitor is currently running
    fn is_running(&self) -> bool;

    /// Get a stream of network connection events
    fn events(&self) -> EventStream<ConnectionInfo>;

    /// Get current active connections (snapshot)
    fn snapshot(&self) -> PlatformResult<Vec<ConnectionInfo>>;
}

/// Trait for file access monitoring implementations
pub trait FileMonitor: Send + Sync {
    /// Start the file monitor
    fn start(&mut self) -> PlatformResult<()>;

    /// Stop the file monitor
    fn stop(&mut self) -> PlatformResult<()>;

    /// Check if the monitor is currently running
    fn is_running(&self) -> bool;

    /// Get a stream of file operation events
    fn events(&self) -> EventStream<FileOpInfo>;

    /// Add a path to watch for file operations
    fn watch_path(&mut self, path: &str) -> PlatformResult<()>;

    /// Remove a path from watching
    fn unwatch_path(&mut self, path: &str) -> PlatformResult<()>;
}

/// Combined platform monitor providing all monitoring capabilities
pub trait PlatformMonitor: Send + Sync {
    /// Get the process monitor
    fn process_monitor(&self) -> &dyn ProcessMonitor;

    /// Get the network monitor
    fn network_monitor(&self) -> &dyn NetworkMonitor;

    /// Get the file monitor
    fn file_monitor(&self) -> &dyn FileMonitor;

    /// Get the platform name
    fn platform_name(&self) -> &'static str;

    /// Check if elevated privileges are available
    fn has_elevated_privileges(&self) -> bool;

    /// Start all monitors
    fn start_all(&mut self) -> PlatformResult<()>;

    /// Stop all monitors
    fn stop_all(&mut self) -> PlatformResult<()>;
}

/// Factory function type for creating platform monitors
pub type PlatformMonitorFactory = fn() -> PlatformResult<Box<dyn PlatformMonitor>>;

/// Get the appropriate platform monitor for the current OS
#[cfg(target_os = "linux")]
pub fn create_platform_monitor() -> PlatformResult<Box<dyn PlatformMonitor>> {
    // Will be implemented in roea-agent with eBPF support
    Err(PlatformError::NotSupported(
        "Linux monitor not yet initialized - use roea-agent".to_string(),
    ))
}

#[cfg(target_os = "macos")]
pub fn create_platform_monitor() -> PlatformResult<Box<dyn PlatformMonitor>> {
    Err(PlatformError::NotSupported(
        "macOS Endpoint Security monitor not yet implemented".to_string(),
    ))
}

#[cfg(target_os = "windows")]
pub fn create_platform_monitor() -> PlatformResult<Box<dyn PlatformMonitor>> {
    Err(PlatformError::NotSupported(
        "Windows ETW monitor not yet implemented".to_string(),
    ))
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn create_platform_monitor() -> PlatformResult<Box<dyn PlatformMonitor>> {
    Err(PlatformError::NotSupported(
        "Unsupported platform".to_string(),
    ))
}
