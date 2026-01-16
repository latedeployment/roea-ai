//! roea-agent - AI Agent Monitoring Daemon
//!
//! A cross-platform daemon that monitors AI coding agents and provides
//! telemetry data via gRPC to the roea-ai UI.
//!
//! # TUI Mode
//!
//! Run with `--tui` flag for a modern terminal interface that displays
//! real-time monitoring without opening a network port.
//!
//! # File Protection
//!
//! Use `--protect-config <file.toml>` to monitor sensitive files and
//! generate alerts when AI agents access them.

mod file;
mod grpc;
mod monitor;
mod network;
mod osquery;
pub mod protection;
mod storage;
mod telemetry;
mod tui;

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::Local;
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
    /// OTLP endpoint for telemetry export
    otlp_endpoint: Option<String>,
    /// Enable telemetry
    telemetry_enabled: bool,
    /// Show live events for tracked AI agents (legacy text mode)
    show_events: bool,
    /// Enable TUI mode (modern terminal interface)
    tui_mode: bool,
    /// Path to protection config file
    protect_config: Option<PathBuf>,
    /// Generate example protection config and exit
    gen_protect_config: bool,
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
            otlp_endpoint: None,
            telemetry_enabled: true,
            show_events: false,
            tui_mode: false,
            protect_config: None,
            gen_protect_config: false,
        }
    }
}

impl Config {
    /// Load configuration from environment, defaults, and command line args
    fn from_env() -> Self {
        let mut config = Self::default();

        // Parse command line arguments
        let args: Vec<String> = std::env::args().collect();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--show-events" | "-e" => {
                    config.show_events = true;
                }
                "--tui" | "-t" => {
                    config.tui_mode = true;
                }
                "--listen" | "-l" => {
                    if let Some(addr) = args.get(i + 1) {
                        if let Ok(parsed) = addr.parse() {
                            config.listen_addr = parsed;
                        }
                        i += 1;
                    }
                }
                "--protect-config" | "-p" => {
                    if let Some(path) = args.get(i + 1) {
                        config.protect_config = Some(PathBuf::from(path));
                        i += 1;
                    }
                }
                "--gen-protect-config" => {
                    config.gen_protect_config = true;
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => {}
            }
            i += 1;
        }

        // Environment variables (can override CLI)
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

        if let Ok(endpoint) = std::env::var("ROEA_OTLP_ENDPOINT") {
            config.otlp_endpoint = Some(endpoint);
        }

        if let Ok(enabled) = std::env::var("ROEA_TELEMETRY_ENABLED") {
            config.telemetry_enabled = enabled.parse().unwrap_or(true);
        }

        if let Ok(path) = std::env::var("ROEA_PROTECT_CONFIG") {
            config.protect_config = Some(PathBuf::from(path));
        }

        config
    }
}

fn print_help() {
    println!("roea-agent - AI Agent Monitoring Daemon");
    println!();
    println!("A cross-platform daemon that monitors AI coding agents and provides");
    println!("real-time observability through a modern TUI or gRPC API.");
    println!();
    println!("USAGE:");
    println!("    roea-agent [OPTIONS]");
    println!();
    println!("DISPLAY MODES:");
    println!("    -t, --tui                   Modern terminal UI (no network port)");
    println!("    -e, --show-events           Simple text event stream");
    println!("    (default)                   gRPC server mode for UI connection");
    println!();
    println!("OPTIONS:");
    println!("    -l, --listen <ADDR>         Address to bind gRPC server (default: 127.0.0.1:50051)");
    println!("    -p, --protect-config <FILE> Path to protection config (TOML)");
    println!("    --gen-protect-config        Generate example protection config and exit");
    println!("    -h, --help                  Print this help message");
    println!();
    println!("FILE PROTECTION:");
    println!("    Monitor sensitive files and alert when AI agents access them.");
    println!("    Use --gen-protect-config to create an example config file.");
    println!();
    println!("    Example:");
    println!("      roea-agent --tui --protect-config ~/.config/roea/protect.toml");
    println!();
    println!("ENVIRONMENT VARIABLES:");
    println!("    ROEA_LISTEN_ADDR        gRPC server address");
    println!("    ROEA_DB_PATH            Database file path");
    println!("    ROEA_LOG_LEVEL          Log level (trace, debug, info, warn, error)");
    println!("    ROEA_OTLP_ENDPOINT      OpenTelemetry OTLP endpoint");
    println!("    ROEA_TELEMETRY_ENABLED  Enable/disable telemetry (true/false)");
    println!("    ROEA_PROTECT_CONFIG     Protection config file path");
    println!();
    println!("EXAMPLES:");
    println!("    roea-agent --tui                    # Run with modern TUI");
    println!("    roea-agent --show-events            # Simple event stream");
    println!("    roea-agent                          # gRPC server (for UI)");
    println!("    roea-agent --tui -p protect.toml    # TUI with file protection");
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::from_env();

    // Handle --gen-protect-config (generate example and exit)
    if config.gen_protect_config {
        println!("{}", protection::ProtectionConfig::example_toml());
        return Ok(());
    }

    // Load protection config if specified
    let protection_config = if let Some(ref path) = config.protect_config {
        match protection::ProtectionConfig::from_file(path) {
            Ok(cfg) => {
                eprintln!("Loaded protection config from {:?} ({} items)", path, cfg.protected_count());
                Some(cfg)
            }
            Err(e) => {
                eprintln!("Error loading protection config: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Use default protection in TUI mode
        if config.tui_mode {
            Some(protection::ProtectionConfig::default())
        } else {
            None
        }
    };

    // In TUI mode, don't initialize logging to file (would interfere with display)
    if !config.tui_mode {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("roea_agent={}", config.log_level).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();

        tracing::info!("Starting roea-agent daemon");
        tracing::info!("Platform: {}", std::env::consts::OS);
        if !config.show_events {
            tracing::info!("Listen address: {}", config.listen_addr);
        }
    }

    // Initialize storage
    let storage_config = StorageConfig {
        db_path: config.db_path.map(|p| p.to_string_lossy().to_string()),
        retention_hours: config.retention_hours,
        cleanup_on_start: true,
    };

    let storage = Storage::new(storage_config).context("Failed to initialize storage")?;
    let storage = Arc::new(storage);

    if !config.tui_mode {
        tracing::info!("Storage initialized");
    }

    // Initialize telemetry if enabled (and not in TUI mode for cleaner display)
    let mut telemetry_service = None;
    if config.telemetry_enabled && !config.tui_mode {
        let telemetry_config = telemetry::TelemetryConfig {
            service_name: "roea-agent".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            otlp_endpoint: config.otlp_endpoint.clone(),
            console_export: false,
            batch_delay_ms: 5000,
            max_batch_size: 512,
        };

        let mut service = telemetry::TelemetryService::new(telemetry_config);
        if let Err(e) = service.init().await {
            tracing::warn!("Failed to initialize telemetry: {} (continuing without it)", e);
        } else {
            tracing::info!("OpenTelemetry telemetry initialized");
            if config.otlp_endpoint.is_some() {
                tracing::info!("OTLP export enabled to: {}", config.otlp_endpoint.as_ref().unwrap());
            }
            telemetry_service = Some(Arc::new(service));
        }
    }

    // Create agent state
    let state = Arc::new(RwLock::new(AgentState::new(storage.clone(), telemetry_service.clone())));

    // Start process monitor
    {
        let mut state_lock = state.write();
        state_lock
            .monitor
            .start()
            .context("Failed to start process monitor")?;
    }
    if !config.tui_mode {
        tracing::info!("Process monitor started");
    }

    // Scan existing processes for AI agents
    {
        let state_lock = state.read();
        state_lock.scan_existing_processes();
    }

    // Start network monitor
    {
        let mut state_lock = state.write();
        if let Err(e) = state_lock.network_monitor.start() {
            if !config.tui_mode {
                tracing::warn!("Network monitor failed to start: {} (continuing without it)", e);
            }
        } else if !config.tui_mode {
            tracing::info!("Network monitor started");
        }
    }

    // Start file monitor
    {
        let mut state_lock = state.write();
        if let Err(e) = state_lock.file_monitor.start() {
            if !config.tui_mode {
                tracing::warn!("File monitor failed to start: {} (continuing without it)", e);
            }
        } else if !config.tui_mode {
            tracing::info!("File monitor started");
        }
    }

    // === MODE SELECTION ===
    // TUI mode: Run modern terminal interface (no network port)
    if config.tui_mode {
        return tui::run_tui(state, protection_config)
            .await
            .map_err(|e| anyhow::anyhow!("TUI error: {}", e));
    }

    // Legacy event streaming mode
    if config.show_events {
        start_event_streaming(state.clone());
        // Keep running forever in event mode
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    }

    // Default: gRPC server mode
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

/// Start background tasks to stream events to stdout
fn start_event_streaming(state: Arc<RwLock<AgentState>>) {
    println!("\n📡 Live event streaming enabled. Showing events for tracked AI agents...\n");
    
    // Get the tracked AI agent PIDs and their info
    let tracked_pids: Arc<RwLock<HashSet<u32>>> = Arc::new(RwLock::new(HashSet::new()));
    let known_processes: Arc<RwLock<HashMap<u32, String>>> = Arc::new(RwLock::new(HashMap::new()));
    let known_connections: Arc<RwLock<HashSet<(u32, String, u16)>>> = Arc::new(RwLock::new(HashSet::new()));
    let known_files: Arc<RwLock<HashSet<(u32, String)>>> = Arc::new(RwLock::new(HashSet::new()));
    
    // Populate initial tracked PIDs
    {
        let state_lock = state.read();
        if let Ok(processes) = state_lock.monitor.snapshot() {
            let mut pids = tracked_pids.write();
            let mut known = known_processes.write();
            for process in &processes {
                if state_lock.signature_matcher.match_process(process).is_some() {
                    pids.insert(process.pid);
                }
                known.insert(process.pid, process.name.clone());
            }
            println!("Tracking {} AI agent processes (polling every 500ms)\n", pids.len());
        }
    }
    
    // Spawn polling task for process changes
    let tracked_pids_clone = tracked_pids.clone();
    let known_processes_clone = known_processes.clone();
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
        loop {
            interval.tick().await;
            
            let state_lock = state_clone.read();
            if let Ok(processes) = state_lock.monitor.snapshot() {
                let mut pids = tracked_pids_clone.write();
                let mut known = known_processes_clone.write();
                let current_pids: HashSet<u32> = processes.iter().map(|p| p.pid).collect();
                
                // Check for new processes
                for process in &processes {
                    if !known.contains_key(&process.pid) {
                        let is_agent = state_lock.signature_matcher.match_process(process).is_some();
                        let agent_type = state_lock.signature_matcher.match_process(process);
                        if is_agent {
                            pids.insert(process.pid);
                        }
                        
                        // Check if parent is tracked
                        let parent_tracked = process.ppid.map(|p| pids.contains(&p)).unwrap_or(false);
                        
                        if is_agent || parent_tracked || pids.contains(&process.pid) {
                            let timestamp = Local::now().format("%H:%M:%S%.3f");
                            
                            // Get a display name - use agent type if detected, otherwise process name
                            // Handle weird names like version numbers (e.g., "2.1.7" from Bun)
                            let display_name = if let Some(agent) = &agent_type {
                                agent.to_string()
                            } else if process.name.chars().all(|c| c.is_ascii_digit() || c == '.') {
                                // Name looks like a version number, try to get from cmdline
                                process.cmdline.as_ref()
                                    .and_then(|c| c.split_whitespace().next())
                                    .and_then(|s| s.rsplit('/').next())
                                    .unwrap_or(&process.name)
                                    .to_string()
                            } else {
                                process.name.clone()
                            };
                            
                            let cmdline_display = process.cmdline.as_deref().unwrap_or("");
                            let agent_marker = if is_agent { " [AI]" } else { "" };
                            
                            // Don't repeat the name in cmdline if they're the same
                            let cmdline_show = if cmdline_display == display_name || cmdline_display.is_empty() {
                                "".to_string()
                            } else {
                                format!(" {}", cmdline_display)
                            };
                            
                            println!(
                                "[{}] 🚀 SPAWN PID:{} {}{}{}",
                                timestamp,
                                process.pid,
                                display_name,
                                cmdline_show,
                                agent_marker
                            );
                        }
                        // Store the better display name for later use in FILE/NET events
                        let stored_name = if let Some(agent) = &agent_type {
                            agent.to_string()
                        } else if process.name.chars().all(|c| c.is_ascii_digit() || c == '.') {
                            process.cmdline.as_ref()
                                .and_then(|c| c.split_whitespace().next())
                                .and_then(|s| s.rsplit('/').next())
                                .unwrap_or(&process.name)
                                .to_string()
                        } else {
                            process.name.clone()
                        };
                        known.insert(process.pid, stored_name);
                    }
                }
                
                // Check for exited processes
                let exited: Vec<(u32, String)> = known.iter()
                    .filter(|(pid, _)| !current_pids.contains(pid))
                    .map(|(pid, name)| (*pid, name.clone()))
                    .collect();
                
                for (pid, name) in exited {
                    if pids.contains(&pid) {
                        let timestamp = Local::now().format("%H:%M:%S%.3f");
                        println!("[{}] 💀 EXIT  PID:{} {}", timestamp, pid, name);
                        pids.remove(&pid);
                    }
                    known.remove(&pid);
                }
            }
            drop(state_lock);
        }
    });
    
    // Spawn polling task for network connections
    let tracked_pids_clone = tracked_pids.clone();
    let known_connections_clone = known_connections.clone();
    let known_processes_clone1 = known_processes.clone();
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(1000));
        loop {
            interval.tick().await;
            
            let state_lock = state_clone.read();
            if let Ok(connections) = state_lock.network_monitor.snapshot() {
                let pids = tracked_pids_clone.read();
                let mut known = known_connections_clone.write();
                let processes = known_processes_clone1.read();
                
                for conn in &connections {
                    if pids.contains(&conn.pid) {
                        let remote_addr = conn.remote_addr.as_deref().unwrap_or("").to_string();
                        let remote_port = conn.remote_port.unwrap_or(0);
                        let key = (conn.pid, remote_addr.clone(), remote_port);
                        
                        if !known.contains(&key) && !remote_addr.is_empty() && remote_port > 0 {
                            known.insert(key);
                            let timestamp = Local::now().format("%H:%M:%S%.3f");
                            let proc_name = processes.get(&conn.pid).map(|s| s.as_str()).unwrap_or("?");
                            let local_addr = conn.local_addr.as_deref().unwrap_or("?");
                            let local_port = conn.local_port.map(|p| p.to_string()).unwrap_or_else(|| "?".to_string());
                            println!(
                                "[{}] 🌐 NET   PID:{} {} {:?} {}:{} → {}:{}",
                                timestamp,
                                conn.pid,
                                proc_name,
                                conn.protocol,
                                local_addr,
                                local_port,
                                remote_addr,
                                remote_port
                            );
                        }
                    }
                }
            }
            drop(state_lock);
        }
    });
    
    // Spawn polling task for file operations
    let tracked_pids_clone = tracked_pids.clone();
    let known_files_clone = known_files.clone();
    let known_processes_clone2 = known_processes.clone();
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(1000));
        loop {
            interval.tick().await;
            
            let state_lock = state_clone.read();
            if let Ok(files) = state_lock.file_monitor.snapshot() {
                let pids = tracked_pids_clone.read();
                let mut known = known_files_clone.write();
                let processes = known_processes_clone2.read();
                
                for file_op in &files {
                    if pids.contains(&file_op.pid) {
                        let key = (file_op.pid, file_op.path.clone());
                        
                        if !known.contains(&key) {
                            known.insert(key);
                            let timestamp = Local::now().format("%H:%M:%S%.3f");
                            let proc_name = processes.get(&file_op.pid).map(|s| s.as_str()).unwrap_or("?");
                            let op_icon = match file_op.operation {
                                roea_common::FileOperation::Open => "📂",
                                roea_common::FileOperation::Read => "📖",
                                roea_common::FileOperation::Write => "✏️ ",
                                roea_common::FileOperation::Create => "🆕",
                                roea_common::FileOperation::Delete => "🗑️ ",
                                roea_common::FileOperation::Rename => "📝",
                            };
                            println!(
                                "[{}] {} FILE  PID:{} {} {:?} {}",
                                timestamp,
                                op_icon,
                                file_op.pid,
                                proc_name,
                                file_op.operation,
                                file_op.path
                            );
                        }
                    }
                }
            }
            drop(state_lock);
        }
    });
}
