//! gRPC server implementation for roea-agent
//!
//! Provides the IPC interface for the UI to communicate with the daemon.

use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use futures_core::Stream;
use parking_lot::RwLock;
use roea_common::{default_signatures, ProcessEventType, SignatureMatcher};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request, Response, Status};

use crate::file::FileMonitorService;
use crate::monitor::ProcessMonitorService;
use crate::network::NetworkMonitorService;
use crate::storage::Storage;
use crate::telemetry::TelemetryService;

// Include generated protobuf code
pub mod proto {
    tonic::include_proto!("roea");
}

use proto::roea_agent_server::{RoeaAgent, RoeaAgentServer};
use proto::*;

/// State shared across the gRPC service
pub struct AgentState {
    pub monitor: ProcessMonitorService,
    pub network_monitor: NetworkMonitorService,
    pub file_monitor: FileMonitorService,
    pub storage: Arc<Storage>,
    pub signature_matcher: SignatureMatcher,
    pub start_time: Instant,
    pub telemetry: Option<Arc<TelemetryService>>,
}

impl AgentState {
    pub fn new(storage: Arc<Storage>, telemetry: Option<Arc<TelemetryService>>) -> Self {
        let mut signature_matcher = SignatureMatcher::new();
        if let Err(e) = signature_matcher.load(default_signatures()) {
            tracing::warn!("Failed to load default signatures: {}", e);
        }

        Self {
            monitor: ProcessMonitorService::new(),
            network_monitor: NetworkMonitorService::new(),
            file_monitor: FileMonitorService::new(),
            storage,
            signature_matcher,
            start_time: Instant::now(),
            telemetry,
        }
    }
}

/// gRPC service implementation
pub struct RoeaAgentService {
    state: Arc<RwLock<AgentState>>,
}

impl RoeaAgentService {
    pub fn new(state: Arc<RwLock<AgentState>>) -> Self {
        Self { state }
    }

    /// Create a gRPC server
    pub fn into_server(self) -> RoeaAgentServer<Self> {
        RoeaAgentServer::new(self)
    }
}

/// Convert internal ProcessInfo to protobuf Process
fn process_info_to_proto(info: &roea_common::ProcessInfo) -> Process {
    Process {
        id: info.id.to_string(),
        pid: info.pid as i32,
        ppid: info.ppid.map(|p| p as i32).unwrap_or(0),
        name: info.name.clone(),
        cmdline: info.cmdline.clone().unwrap_or_default(),
        exe_path: info.exe_path.clone().unwrap_or_default(),
        agent_type: info.agent_type.clone().unwrap_or_default(),
        start_time: info.start_time.timestamp_millis(),
        end_time: info.end_time.map(|t| t.timestamp_millis()).unwrap_or(0),
        user: info.user.clone().unwrap_or_default(),
        cwd: info.cwd.clone().unwrap_or_default(),
        connections: vec![],
        recent_file_ops: vec![],
    }
}

#[tonic::async_trait]
impl RoeaAgent for RoeaAgentService {
    type WatchProcessesStream =
        Pin<Box<dyn Stream<Item = Result<ProcessEvent, Status>> + Send + 'static>>;

    async fn watch_processes(
        &self,
        request: Request<WatchRequest>,
    ) -> Result<Response<Self::WatchProcessesStream>, Status> {
        let req = request.into_inner();
        let state = self.state.read();

        // Get event receiver
        let rx = state.monitor.subscribe();

        // If include_existing, send current processes first
        let existing_processes = if req.include_existing {
            state
                .monitor
                .snapshot()
                .map_err(|e| Status::internal(format!("Failed to get snapshot: {}", e)))?
        } else {
            vec![]
        };

        let agent_filter: Vec<String> = req.agent_types;
        let signature_matcher = SignatureMatcher::new(); // Clone would be better but not impl

        // Create stream that first emits existing processes, then new events
        let stream = async_stream::stream! {
            // Emit existing processes first
            for mut process in existing_processes {
                // Apply signature matching
                let matcher = SignatureMatcher::new();
                // Note: In production, we'd share the matcher properly
                if let Some(_agent_type) = process.agent_type.as_ref() {
                    // Already has agent type
                } else {
                    // Would apply matcher here
                }

                if !agent_filter.is_empty() {
                    if let Some(ref agent_type) = process.agent_type {
                        if !agent_filter.contains(agent_type) {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }

                yield Ok(ProcessEvent {
                    event_type: process_event::EventType::Spawn as i32,
                    process: Some(process_info_to_proto(&process)),
                    timestamp: Utc::now().timestamp_millis(),
                });
            }

            // Then stream new events
            let mut rx_stream = BroadcastStream::new(rx);
            use tokio_stream::StreamExt;

            while let Some(event_result) = rx_stream.next().await {
                match event_result {
                    Ok(event) => {
                        // Filter by agent type if specified
                        if !agent_filter.is_empty() {
                            if let Some(ref agent_type) = event.process.agent_type {
                                if !agent_filter.contains(agent_type) {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        }

                        let event_type = match event.event_type {
                            ProcessEventType::Spawn => process_event::EventType::Spawn,
                            ProcessEventType::Exit => process_event::EventType::Exit,
                            ProcessEventType::Update => process_event::EventType::Update,
                        };

                        yield Ok(ProcessEvent {
                            event_type: event_type as i32,
                            process: Some(process_info_to_proto(&event.process)),
                            timestamp: event.timestamp.timestamp_millis(),
                        });
                    }
                    Err(_) => {
                        // Lagged or closed
                        break;
                    }
                }
            }
        };

        Ok(Response::new(Box::pin(stream)))
    }

    async fn query_processes(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        let req = request.into_inner();
        let state = self.state.read();

        let start_time = if req.start_time > 0 {
            chrono::DateTime::from_timestamp_millis(req.start_time)
        } else {
            None
        };

        let end_time = if req.end_time > 0 {
            chrono::DateTime::from_timestamp_millis(req.end_time)
        } else {
            None
        };

        let agent_type = req.agent_types.first().map(|s| s.as_str());
        let limit = if req.limit > 0 { req.limit as usize } else { 100 };
        let offset = req.offset as usize;

        let processes = state
            .storage
            .query_processes(start_time, end_time, agent_type, limit + 1, offset)
            .map_err(|e| Status::internal(format!("Query failed: {}", e)))?;

        let has_more = processes.len() > limit;
        let processes: Vec<Process> = processes
            .into_iter()
            .take(limit)
            .map(|p| process_info_to_proto(&p))
            .collect();

        Ok(Response::new(QueryResponse {
            processes,
            total_count: 0, // Would need separate count query
            has_more,
        }))
    }

    async fn query_connections(
        &self,
        _request: Request<QueryRequest>,
    ) -> Result<Response<ConnectionsResponse>, Status> {
        let state = self.state.read();

        // Get current connections snapshot
        let conn_infos = state
            .network_monitor
            .snapshot()
            .map_err(|e| Status::internal(format!("Failed to get connections: {}", e)))?;

        let connections: Vec<Connection> = conn_infos
            .into_iter()
            .filter(|c| c.pid > 0) // Only include connections with known PIDs
            .map(|c| Connection {
                id: c.id.to_string(),
                process_id: c.process_id.map(|id| id.to_string()).unwrap_or_default(),
                pid: c.pid as i32,
                protocol: format!("{:?}", c.protocol).to_lowercase(),
                local_addr: c.local_addr.unwrap_or_default(),
                local_port: c.local_port.map(|p| p as i32).unwrap_or(0),
                remote_addr: c.remote_addr.unwrap_or_default(),
                remote_port: c.remote_port.map(|p| p as i32).unwrap_or(0),
                state: format!("{:?}", c.state).to_lowercase(),
                timestamp: c.timestamp.timestamp_millis(),
            })
            .collect();

        Ok(Response::new(ConnectionsResponse {
            connections,
            total_count: 0,
            has_more: false,
        }))
    }

    async fn query_file_ops(
        &self,
        _request: Request<QueryRequest>,
    ) -> Result<Response<FileOpsResponse>, Status> {
        let state = self.state.read();

        // Get current open files snapshot
        let file_infos = state
            .file_monitor
            .snapshot()
            .map_err(|e| Status::internal(format!("Failed to get file ops: {}", e)))?;

        let file_ops: Vec<FileOp> = file_infos
            .into_iter()
            .filter(|f| f.pid > 0)
            .map(|f| FileOp {
                id: f.id.to_string(),
                process_id: f.process_id.map(|id| id.to_string()).unwrap_or_default(),
                pid: f.pid as i32,
                operation: format!("{:?}", f.operation).to_lowercase(),
                path: f.path,
                new_path: f.new_path.unwrap_or_default(),
                timestamp: f.timestamp.timestamp_millis(),
            })
            .collect();

        Ok(Response::new(FileOpsResponse {
            file_ops,
            total_count: 0,
            has_more: false,
        }))
    }

    async fn get_agent_signatures(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<SignaturesResponse>, Status> {
        let state = self.state.read();

        let signatures: Vec<AgentSignature> = state
            .signature_matcher
            .signatures()
            .map(|sig| AgentSignature {
                name: sig.name.clone(),
                display_name: sig.display_name.clone(),
                icon: sig.icon.clone().unwrap_or_default(),
                expected_endpoints: sig.network_endpoints.expected.clone(),
                child_process_tracking: sig.child_process_tracking,
            })
            .collect();

        Ok(Response::new(SignaturesResponse { signatures }))
    }

    async fn get_status(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<StatusResponse>, Status> {
        let state = self.state.read();

        let uptime = state.start_time.elapsed().as_secs() as i64;
        let processes_tracked = state
            .storage
            .process_count()
            .unwrap_or(0);
        let events_collected = state
            .storage
            .total_event_count()
            .unwrap_or(0);

        Ok(Response::new(StatusResponse {
            running: state.monitor.is_running(),
            platform: std::env::consts::OS.to_string(),
            elevated_privileges: is_elevated(),
            uptime_seconds: uptime,
            processes_tracked,
            events_collected,
        }))
    }
}

/// Check if running with elevated privileges
fn is_elevated() -> bool {
    #[cfg(unix)]
    {
        // Check if running as root or has CAP_SYS_ADMIN
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(windows)]
    {
        // Would check for admin privileges
        false
    }
    #[cfg(not(any(unix, windows)))]
    {
        false
    }
}
