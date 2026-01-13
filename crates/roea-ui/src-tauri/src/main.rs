//! roea-ai Desktop Application
//!
//! Tauri-based cross-platform desktop UI for monitoring AI coding agents.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod grpc_client;

use std::sync::Arc;

use tauri::{Manager, State};
use tokio::sync::RwLock;

use grpc_client::AgentClient;

/// Application state shared across the Tauri app
pub struct AppState {
    client: Arc<RwLock<Option<AgentClient>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
        }
    }
}

/// Connect to the roea-agent daemon
#[tauri::command]
async fn connect_to_agent(
    state: State<'_, AppState>,
    address: Option<String>,
) -> Result<bool, String> {
    let addr = address.unwrap_or_else(|| "http://127.0.0.1:50051".to_string());

    match AgentClient::connect(&addr).await {
        Ok(client) => {
            let mut guard = state.client.write().await;
            *guard = Some(client);
            Ok(true)
        }
        Err(e) => Err(format!("Failed to connect: {}", e)),
    }
}

/// Get agent daemon status
#[tauri::command]
async fn get_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let guard = state.client.read().await;
    let client = guard.as_ref().ok_or("Not connected to agent")?;

    client
        .get_status()
        .await
        .map_err(|e| format!("Failed to get status: {}", e))
}

/// Get current process snapshot
#[tauri::command]
async fn get_processes(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let guard = state.client.read().await;
    let client = guard.as_ref().ok_or("Not connected to agent")?;

    client
        .get_processes()
        .await
        .map_err(|e| format!("Failed to get processes: {}", e))
}

/// Get agent signatures
#[tauri::command]
async fn get_signatures(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let guard = state.client.read().await;
    let client = guard.as_ref().ok_or("Not connected to agent")?;

    client
        .get_signatures()
        .await
        .map_err(|e| format!("Failed to get signatures: {}", e))
}

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            connect_to_agent,
            get_status,
            get_processes,
            get_signatures,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
