//! gRPC client for communicating with roea-agent daemon

use anyhow::Result;
use serde_json::{json, Value};
use tonic::transport::Channel;

// Include generated protobuf code
pub mod proto {
    tonic::include_proto!("roea");
}

use proto::roea_agent_client::RoeaAgentClient;
use proto::{Empty, QueryRequest};

/// Client for the roea-agent daemon
#[derive(Clone)]
pub struct AgentClient {
    inner: RoeaAgentClient<Channel>,
}

impl AgentClient {
    /// Connect to the agent daemon
    pub async fn connect(addr: &str) -> Result<Self> {
        let channel = Channel::from_shared(addr.to_string())?
            .connect()
            .await?;

        Ok(Self {
            inner: RoeaAgentClient::new(channel),
        })
    }

    /// Get daemon status
    pub async fn get_status(&self) -> Result<Value> {
        let mut client = self.inner.clone();
        let response = client.get_status(Empty {}).await?;
        let status = response.into_inner();

        Ok(json!({
            "running": status.running,
            "platform": status.platform,
            "elevatedPrivileges": status.elevated_privileges,
            "uptimeSeconds": status.uptime_seconds,
            "processesTracked": status.processes_tracked,
            "eventsCollected": status.events_collected,
        }))
    }

    /// Get current processes
    pub async fn get_processes(&self) -> Result<Vec<Value>> {
        let mut client = self.inner.clone();
        let request = QueryRequest {
            start_time: 0,
            end_time: 0,
            agent_types: vec![],
            process_name_pattern: String::new(),
            limit: 100,
            offset: 0,
        };

        let response = client.query_processes(request).await?;
        let processes = response.into_inner().processes;

        Ok(processes
            .into_iter()
            .map(|p| {
                json!({
                    "id": p.id,
                    "pid": p.pid,
                    "ppid": p.ppid,
                    "name": p.name,
                    "cmdline": p.cmdline,
                    "exePath": p.exe_path,
                    "agentType": p.agent_type,
                    "startTime": p.start_time,
                    "endTime": p.end_time,
                    "user": p.user,
                    "cwd": p.cwd,
                })
            })
            .collect())
    }

    /// Get agent signatures
    pub async fn get_signatures(&self) -> Result<Vec<Value>> {
        let mut client = self.inner.clone();
        let response = client.get_agent_signatures(Empty {}).await?;
        let signatures = response.into_inner().signatures;

        Ok(signatures
            .into_iter()
            .map(|s| {
                json!({
                    "name": s.name,
                    "displayName": s.display_name,
                    "icon": s.icon,
                    "expectedEndpoints": s.expected_endpoints,
                    "childProcessTracking": s.child_process_tracking,
                })
            })
            .collect())
    }
}
