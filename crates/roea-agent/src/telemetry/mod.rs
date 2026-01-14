//! OpenTelemetry integration for roea-ai
//!
//! This module provides standardized telemetry collection and export using
//! the OpenTelemetry specification. It enables:
//! - Distributed tracing for AI agent activities
//! - Metrics collection for agent performance
//! - OTLP export for integration with observability platforms

use std::time::Duration;

use opentelemetry::{
    global,
    trace::{Span, SpanKind, Tracer, TracerProvider as _},
    KeyValue,
};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    runtime::Tokio,
    trace::{BatchSpanProcessor, TracerProvider},
    Resource,
};
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};
use thiserror::Error;
use tracing::{debug, info};

use roea_common::events::{ConnectionInfo, FileOpInfo, ProcessInfo};

/// Telemetry configuration
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Service name for OTEL resource
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// OTLP endpoint (e.g., "http://localhost:4317")
    pub otlp_endpoint: Option<String>,
    /// Enable console exporter for debugging
    pub console_export: bool,
    /// Batch export delay in milliseconds
    pub batch_delay_ms: u64,
    /// Maximum batch size
    pub max_batch_size: usize,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            service_name: "roea-agent".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            otlp_endpoint: None,
            console_export: false,
            batch_delay_ms: 5000,
            max_batch_size: 512,
        }
    }
}

/// Telemetry errors
#[derive(Error, Debug)]
pub enum TelemetryError {
    #[error("Failed to initialize tracer: {0}")]
    TracerInit(String),
    #[error("Failed to create exporter: {0}")]
    ExporterError(String),
    #[error("Telemetry not initialized")]
    NotInitialized,
}

/// OpenTelemetry telemetry service
pub struct TelemetryService {
    config: TelemetryConfig,
    tracer: Option<opentelemetry_sdk::trace::Tracer>,
}

impl TelemetryService {
    /// Create a new telemetry service
    pub fn new(config: TelemetryConfig) -> Self {
        Self {
            config,
            tracer: None,
        }
    }

    /// Initialize the telemetry service
    pub async fn init(&mut self) -> Result<(), TelemetryError> {
        let resource = Resource::new(vec![
            KeyValue::new(SERVICE_NAME, self.config.service_name.clone()),
            KeyValue::new(SERVICE_VERSION, self.config.service_version.clone()),
            KeyValue::new("service.namespace", "roea-ai"),
            KeyValue::new("deployment.environment", "local"),
        ]);

        let mut provider_builder = TracerProvider::builder().with_resource(resource);

        // Add OTLP exporter if configured
        if let Some(endpoint) = &self.config.otlp_endpoint {
            info!("Initializing OTLP exporter to {}", endpoint);

            let exporter = SpanExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint)
                .with_timeout(Duration::from_secs(10))
                .build()
                .map_err(|e| TelemetryError::ExporterError(e.to_string()))?;

            let batch_processor = BatchSpanProcessor::builder(exporter, Tokio)
                .with_batch_config(
                    opentelemetry_sdk::trace::BatchConfigBuilder::default()
                        .with_scheduled_delay(Duration::from_millis(self.config.batch_delay_ms))
                        .with_max_export_batch_size(self.config.max_batch_size)
                        .build(),
                )
                .build();

            provider_builder = provider_builder.with_span_processor(batch_processor);
        }

        let provider = provider_builder.build();
        let tracer = provider.tracer("roea-agent");

        // Set global provider for cross-crate usage
        global::set_tracer_provider(provider);

        self.tracer = Some(tracer);

        info!("OpenTelemetry telemetry service initialized");
        Ok(())
    }

    /// Shutdown the telemetry service
    pub fn shutdown(&self) {
        info!("Shutting down telemetry service");
        global::shutdown_tracer_provider();
    }

    /// Record a process spawn event
    pub fn record_process_spawn(&self, process: &ProcessInfo) {
        let Some(tracer) = &self.tracer else {
            return;
        };

        let mut span = tracer
            .span_builder(format!("process.spawn.{}", process.name))
            .with_kind(SpanKind::Internal)
            .with_attributes(vec![
                KeyValue::new("process.pid", process.pid as i64),
                KeyValue::new("process.parent_pid", process.ppid.map(|p| p as i64).unwrap_or(0)),
                KeyValue::new("process.executable.name", process.name.clone()),
                KeyValue::new("process.executable.path", process.exe_path.clone().unwrap_or_default()),
                KeyValue::new("process.command_line", process.cmdline.clone().unwrap_or_default()),
                KeyValue::new("process.owner", process.user.clone().unwrap_or_default()),
                KeyValue::new("process.working_directory", process.cwd.clone().unwrap_or_default()),
            ])
            .start(tracer);

        // Add agent-specific attributes if this is an AI agent
        if let Some(ref agent_type) = process.agent_type {
            span.set_attribute(KeyValue::new("ai.agent.type", agent_type.clone()));
            span.set_attribute(KeyValue::new("ai.agent.detected", true));
        } else {
            span.set_attribute(KeyValue::new("ai.agent.detected", false));
        }

        span.end();
        debug!("Recorded process spawn: {} (PID: {})", process.name, process.pid);
    }

    /// Record a process exit event
    pub fn record_process_exit(&self, process: &ProcessInfo) {
        let Some(tracer) = &self.tracer else {
            return;
        };

        let mut span = tracer
            .span_builder(format!("process.exit.{}", process.name))
            .with_kind(SpanKind::Internal)
            .with_attributes(vec![
                KeyValue::new("process.pid", process.pid as i64),
                KeyValue::new("process.parent_pid", process.ppid.map(|p| p as i64).unwrap_or(0)),
                KeyValue::new("process.executable.name", process.name.clone()),
            ])
            .start(tracer);

        span.end();
        debug!("Recorded process exit: {} (PID: {})", process.name, process.pid);
    }

    /// Record a network connection event
    pub fn record_connection(&self, conn: &ConnectionInfo, process_name: &str) {
        let Some(tracer) = &self.tracer else {
            return;
        };

        let span_name = format!("network.{:?}.{:?}", conn.protocol, conn.state);

        let mut span = tracer
            .span_builder(span_name)
            .with_kind(SpanKind::Client)
            .with_attributes(vec![
                KeyValue::new("network.transport", format!("{:?}", conn.protocol)),
                KeyValue::new("network.local.address", conn.local_addr.clone().unwrap_or_default()),
                KeyValue::new("network.local.port", conn.local_port.map(|p| p as i64).unwrap_or(0)),
                KeyValue::new("network.peer.address", conn.remote_addr.clone().unwrap_or_default()),
                KeyValue::new("network.peer.port", conn.remote_port.map(|p| p as i64).unwrap_or(0)),
                KeyValue::new("network.connection.state", format!("{:?}", conn.state)),
                KeyValue::new("process.pid", conn.pid as i64),
                KeyValue::new("process.executable.name", process_name.to_string()),
            ])
            .start(tracer);

        // Classify the endpoint
        if let (Some(remote_addr), Some(remote_port)) = (&conn.remote_addr, conn.remote_port) {
            if let Some(classification) = classify_endpoint(remote_addr, remote_port) {
                span.set_attribute(KeyValue::new("network.endpoint.classification", classification));
            }
        }

        span.end();
        debug!(
            "Recorded connection: {:?}:{:?} -> {:?}:{:?}",
            conn.local_addr, conn.local_port, conn.remote_addr, conn.remote_port
        );
    }

    /// Record a file operation event
    pub fn record_file_op(&self, file_op: &FileOpInfo, process_name: &str) {
        let Some(tracer) = &self.tracer else {
            return;
        };

        let span_name = format!("file.{:?}", file_op.operation);

        let mut attributes = vec![
            KeyValue::new("file.path", file_op.path.clone()),
            KeyValue::new("file.operation", format!("{:?}", file_op.operation)),
            KeyValue::new("process.pid", file_op.pid as i64),
            KeyValue::new("process.executable.name", process_name.to_string()),
        ];

        // Add new path for rename operations
        if let Some(ref new_path) = file_op.new_path {
            if !new_path.is_empty() {
                attributes.push(KeyValue::new("file.new_path", new_path.clone()));
            }
        }

        let mut span = tracer
            .span_builder(span_name)
            .with_kind(SpanKind::Internal)
            .with_attributes(attributes)
            .start(tracer);

        // Classify the file path
        if let Some(classification) = classify_file_path(&file_op.path) {
            span.set_attribute(KeyValue::new("file.classification", classification));
        }

        span.end();
        debug!("Recorded file op: {:?} on {}", file_op.operation, file_op.path);
    }

    /// Record an AI agent session start
    pub fn start_agent_session(&self, process: &ProcessInfo) -> Option<AgentSessionSpan> {
        let tracer = self.tracer.as_ref()?;
        let agent_type = process.agent_type.as_ref()?;

        let span = tracer
            .span_builder(format!("ai.agent.session.{}", agent_type))
            .with_kind(SpanKind::Server)
            .with_attributes(vec![
                KeyValue::new("ai.agent.type", agent_type.clone()),
                KeyValue::new("ai.agent.session.start_time", process.start_time.timestamp()),
                KeyValue::new("process.pid", process.pid as i64),
                KeyValue::new("process.executable.name", process.name.clone()),
                KeyValue::new("process.executable.path", process.exe_path.clone().unwrap_or_default()),
            ])
            .start(tracer);

        info!("Started agent session tracking for {} (PID: {})", agent_type, process.pid);

        Some(AgentSessionSpan {
            span,
            agent_type: agent_type.clone(),
            pid: process.pid,
        })
    }
}

/// Represents an active AI agent session span
pub struct AgentSessionSpan {
    span: opentelemetry_sdk::trace::Span,
    pub agent_type: String,
    pub pid: u32,
}

impl AgentSessionSpan {
    /// Record activity within this session
    pub fn record_activity(&mut self, activity_type: &str, details: &str) {
        self.span.add_event(
            activity_type.to_string(),
            vec![
                KeyValue::new("details", details.to_string()),
                KeyValue::new("timestamp", chrono::Utc::now().timestamp()),
            ],
        );
    }

    /// End the session
    pub fn end(mut self) {
        info!("Ending agent session for {} (PID: {})", self.agent_type, self.pid);
        self.span.end();
    }
}

/// Classify network endpoint based on address and port
fn classify_endpoint(addr: &str, port: u16) -> Option<String> {
    // Known LLM API endpoints
    let llm_apis = [
        ("api.anthropic.com", "anthropic_api"),
        ("api.openai.com", "openai_api"),
        ("api.cohere.com", "cohere_api"),
        ("api.together.xyz", "together_api"),
        ("generativelanguage.googleapis.com", "google_ai_api"),
    ];

    for (domain, classification) in llm_apis {
        if addr.contains(domain) {
            return Some(classification.to_string());
        }
    }

    // GitHub endpoints
    if addr.contains("github.com") || addr.contains("githubusercontent.com") {
        return Some("github".to_string());
    }

    // Package registries
    let registries = [
        ("npmjs.org", "npm_registry"),
        ("registry.yarnpkg.com", "yarn_registry"),
        ("pypi.org", "pypi_registry"),
        ("crates.io", "crates_registry"),
    ];

    for (domain, classification) in registries {
        if addr.contains(domain) {
            return Some(classification.to_string());
        }
    }

    // Standard HTTPS/HTTP
    match port {
        443 => Some("https".to_string()),
        80 => Some("http".to_string()),
        _ => None,
    }
}

/// Classify file path based on patterns
fn classify_file_path(path: &str) -> Option<String> {
    // Source code files
    let code_extensions = [".rs", ".ts", ".tsx", ".js", ".jsx", ".py", ".go", ".java"];
    for ext in code_extensions {
        if path.ends_with(ext) {
            return Some("source_code".to_string());
        }
    }

    // Config files
    let config_patterns = [
        "Cargo.toml",
        "package.json",
        "tsconfig.json",
        ".eslintrc",
        "pyproject.toml",
        ".gitignore",
    ];
    for pattern in config_patterns {
        if path.contains(pattern) {
            return Some("config_file".to_string());
        }
    }

    // Git files
    if path.contains(".git/") {
        return Some("git_internal".to_string());
    }

    // Temporary/cache files
    if path.contains("/tmp/") || path.contains("/cache/") || path.contains("node_modules") {
        return Some("temp_or_cache".to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_endpoint() {
        assert_eq!(
            classify_endpoint("api.anthropic.com", 443),
            Some("anthropic_api".to_string())
        );
        assert_eq!(
            classify_endpoint("github.com", 443),
            Some("github".to_string())
        );
        assert_eq!(
            classify_endpoint("npmjs.org", 443),
            Some("npm_registry".to_string())
        );
        assert_eq!(
            classify_endpoint("192.168.1.1", 443),
            Some("https".to_string())
        );
    }

    #[test]
    fn test_classify_file_path() {
        assert_eq!(
            classify_file_path("/home/user/project/src/main.rs"),
            Some("source_code".to_string())
        );
        assert_eq!(
            classify_file_path("/home/user/project/Cargo.toml"),
            Some("config_file".to_string())
        );
        assert_eq!(
            classify_file_path("/home/user/project/.git/objects/abc123"),
            Some("git_internal".to_string())
        );
    }
}
