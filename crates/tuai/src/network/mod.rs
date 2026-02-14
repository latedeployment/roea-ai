//! Network connection monitoring
//!
//! Tracks TCP/UDP connections and correlates them with processes.

mod proc_net;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use tuai_common::{ConnectionInfo, PlatformResult};
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
        // Check against known API endpoints (including local LLM servers)
        if remote_addr.contains("api.anthropic.com")
            || remote_addr.contains("api.openai.com")
            || remote_addr.contains("api.cursor.sh")
            || remote_addr.contains("api.groq.com")
            || remote_addr.contains("api.together.xyz")
            || remote_addr.contains("api.mistral.ai")
            || remote_addr.contains("generativelanguage.googleapis.com")
        {
            EndpointClass::LlmApi
        } else if Self::is_local_llm_endpoint(remote_addr) {
            EndpointClass::LocalLlm
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

    /// Check if an endpoint is a local LLM server (Ollama, LM Studio, LocalAI, etc.)
    fn is_local_llm_endpoint(remote_addr: &str) -> bool {
        // Common local LLM server ports
        let local_llm_ports = [
            "11434", // Ollama default port
            "1234",  // LM Studio default port
            "8080",  // LocalAI, vLLM default ports
            "5000",  // Various local servers
            "5001",  // Alternative local server port
            "8000",  // Common FastAPI/uvicorn port for local LLMs
            "3000",  // Some local LLM UIs
        ];

        // Check if it's a localhost address with a known LLM port
        let is_localhost = remote_addr.starts_with("127.")
            || remote_addr.starts_with("localhost")
            || remote_addr.starts_with("0.0.0.0")
            || remote_addr == "::1";

        if is_localhost {
            for port in local_llm_ports {
                if remote_addr.contains(&format!(":{}", port)) {
                    return true;
                }
            }
        }

        // Also check for common local LLM hostnames
        remote_addr.contains("ollama")
            || remote_addr.contains("lmstudio")
            || remote_addr.contains("localai")
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
    /// Local LLM servers (Ollama, LM Studio, LocalAI, etc.)
    LocalLlm,
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
