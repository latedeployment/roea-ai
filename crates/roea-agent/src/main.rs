//! roea-agent - AI Agent Monitoring Daemon
//!
//! A cross-platform daemon that monitors AI coding agents and provides
//! telemetry data via gRPC to the roea-ai UI.

mod file;
mod grpc;
mod monitor;
mod network;
mod storage;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use parking_lot::RwLock;
use tonic::transport::Server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::grpc::{AgentState, RoeaAgentService};
use crate::storage::{Storage, StorageConfig};

/// Default gRPC server address
const DEFAULT_ADDR: &str = "127.0.0.1:50051";

/// Configuration for the agent daemon
#[derive(Debug, Clone)]
struct Config {
    /// Address to bind the gRPC server
    listen_addr: SocketAddr,
    /// Path to the database file
    db_path: Option<PathBuf>,
    /// Data retention in hours
    retention_hours: u32,
    /// Log level
    log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        let db_path = ProjectDirs::from("ai", "roea", "roea-agent")
            .map(|dirs| dirs.data_dir().join("telemetry.db"));

        Self {
            listen_addr: DEFAULT_ADDR.parse().unwrap(),
            db_path,
            retention_hours: 24,
            log_level: "info".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from environment and defaults
    fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(addr) = std::env::var("ROEA_LISTEN_ADDR") {
            if let Ok(parsed) = addr.parse() {
                config.listen_addr = parsed;
            }
        }

        if let Ok(path) = std::env::var("ROEA_DB_PATH") {
            config.db_path = Some(PathBuf::from(path));
        }

        if let Ok(hours) = std::env::var("ROEA_RETENTION_HOURS") {
            if let Ok(parsed) = hours.parse() {
                config.retention_hours = parsed;
            }
        }

        if let Ok(level) = std::env::var("ROEA_LOG_LEVEL") {
            config.log_level = level;
        }

        config
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::from_env();

    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("roea_agent={}", config.log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting roea-agent daemon");
    tracing::info!("Platform: {}", std::env::consts::OS);
    tracing::info!("Listen address: {}", config.listen_addr);

    // Initialize storage
    let storage_config = StorageConfig {
        db_path: config.db_path.map(|p| p.to_string_lossy().to_string()),
        retention_hours: config.retention_hours,
        cleanup_on_start: true,
    };

    let storage = Storage::new(storage_config).context("Failed to initialize storage")?;
    let storage = Arc::new(storage);

    tracing::info!("Storage initialized");

    // Create agent state
    let state = Arc::new(RwLock::new(AgentState::new(storage.clone())));

    // Start process monitor
    {
        let mut state_lock = state.write();
        state_lock
            .monitor
            .start()
            .context("Failed to start process monitor")?;
    }
    tracing::info!("Process monitor started");

    // Start network monitor
    {
        let mut state_lock = state.write();
        if let Err(e) = state_lock.network_monitor.start() {
            tracing::warn!("Network monitor failed to start: {} (continuing without it)", e);
        } else {
            tracing::info!("Network monitor started");
        }
    }

    // Start file monitor
    {
        let mut state_lock = state.write();
        if let Err(e) = state_lock.file_monitor.start() {
            tracing::warn!("File monitor failed to start: {} (continuing without it)", e);
        } else {
            tracing::info!("File monitor started");
        }
    }

    // Create gRPC service
    let service = RoeaAgentService::new(state.clone());

    // Start gRPC server
    tracing::info!("Starting gRPC server on {}", config.listen_addr);

    Server::builder()
        .add_service(service.into_server())
        .serve(config.listen_addr)
        .await
        .context("gRPC server failed")?;

    Ok(())
}
