//! Network connection monitoring
//!
//! Tracks TCP/UDP connections and correlates them with processes.

mod proc_net;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::RwLock;
use roea_common::{ConnectionInfo, ConnectionState, PlatformResult, Protocol};
use tokio::sync::broadcast;

pub use proc_net::ProcNetMonitor;

/// Network monitor service
pub struct NetworkMonitorService {
    inner: Box<dyn NetworkMonitorBackend>,
    event_tx: broadcast::Sender<ConnectionInfo>,
    /// Cache of known connections by (pid, remote_addr, remote_port)
    connection_cache: Arc<RwLock<HashMap<(u32, String, u16), ConnectionInfo>>>,
}

impl NetworkMonitorService {
    /// Create a new network monitor service
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        Self {
            inner: Box::new(ProcNetMonitor::new()),
            event_tx,
            connection_cache: Arc::new(RwLock::new(HashMap::new())),
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

    /// Get current connections snapshot
    pub fn snapshot(&self) -> PlatformResult<Vec<ConnectionInfo>> {
        self.inner.snapshot()
    }

    /// Get connections for a specific PID
    pub fn connections_for_pid(&self, pid: u32) -> PlatformResult<Vec<ConnectionInfo>> {
        let all = self.snapshot()?;
        Ok(all.into_iter().filter(|c| c.pid == pid).collect())
    }

    /// Subscribe to connection events
    pub fn subscribe(&self) -> broadcast::Receiver<ConnectionInfo> {
        self.event_tx.subscribe()
    }

    /// Check if a remote address is a known API endpoint
    pub fn classify_endpoint(remote_addr: &str) -> EndpointClass {
        // Check against known API endpoints
        if remote_addr.contains("api.anthropic.com")
            || remote_addr.contains("api.openai.com")
            || remote_addr.contains("api.cursor.sh")
        {
            EndpointClass::LlmApi
        } else if remote_addr.contains("github.com")
            || remote_addr.contains("api.github.com")
            || remote_addr.contains("githubusercontent.com")
        {
            EndpointClass::GitHub
        } else if remote_addr.contains("npmjs.org")
            || remote_addr.contains("registry.npmjs.org")
            || remote_addr.contains("pypi.org")
            || remote_addr.contains("crates.io")
        {
            EndpointClass::PackageRegistry
        } else if remote_addr.contains("sentry.io")
            || remote_addr.contains("statsig")
            || remote_addr.contains("amplitude")
        {
            EndpointClass::Telemetry
        } else if remote_addr.starts_with("127.")
            || remote_addr.starts_with("localhost")
            || remote_addr == "::1"
        {
            EndpointClass::Localhost
        } else {
            EndpointClass::Unknown
        }
    }
}

impl Default for NetworkMonitorService {
    fn default() -> Self {
        Self::new()
    }
}

/// Classification of network endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointClass {
    /// LLM API endpoints (Anthropic, OpenAI, etc.)
    LlmApi,
    /// GitHub API and related
    GitHub,
    /// Package registries (npm, pypi, crates.io)
    PackageRegistry,
    /// Telemetry/analytics
    Telemetry,
    /// Localhost connections
    Localhost,
    /// Unknown external endpoint
    Unknown,
}

/// Trait for network monitoring backends
pub trait NetworkMonitorBackend: Send + Sync {
    fn start(&mut self) -> PlatformResult<()>;
    fn stop(&mut self) -> PlatformResult<()>;
    fn is_running(&self) -> bool;
    fn snapshot(&self) -> PlatformResult<Vec<ConnectionInfo>>;
}
