//! eBPF-based Process Monitor for Linux
//!
//! Provides high-performance, real-time process monitoring using Linux eBPF.
//! This monitor hooks into kernel tracepoints for process exec and exit events,
//! providing immediate notification when AI agents or their child processes
//! are spawned or terminated.
//!
//! Requires:
//! - Linux kernel 5.8+ (for ring buffer support)
//! - CAP_BPF or root privileges
//! - BTF (BPF Type Format) support in kernel
//! - vmlinux.h generated from kernel BTF (see build.rs for instructions)
//!
//! This module is only compiled when the ebpf_available cfg flag is set by build.rs,
//! which happens when vmlinux.h is present and the BPF program compiles successfully.

#![cfg(all(target_os = "linux", ebpf_available))]

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use libbpf_rs::{
    skel::{OpenSkel, Skel, SkelBuilder},
    MapCore, RingBufferBuilder,
};
use parking_lot::RwLock;
use thiserror::Error;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use roea_common::events::ProcessInfo;
use roea_common::{ProcessEvent, ProcessEventType};

// Include generated skeleton
mod process_monitor_skel {
    include!(concat!(env!("OUT_DIR"), "/process_monitor.skel.rs"));
}

use process_monitor_skel::*;

/// Event types from BPF program
const EVENT_PROCESS_EXEC: u32 = 1;
const EVENT_PROCESS_EXIT: u32 = 2;
const EVENT_NETWORK_CONNECT: u32 = 3;
const EVENT_FILE_OPEN: u32 = 4;

/// Address families
const AF_UNIX: u16 = 1;
const AF_INET: u16 = 2;
const AF_INET6: u16 = 10;

/// BPF process event structure (must match C definition)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct BpfProcessEvent {
    event_type: u32,
    pid: u32,
    ppid: u32,
    uid: u32,
    gid: u32,
    timestamp_ns: u64,
    comm: [u8; 256],
    filename: [u8; 256],
    exit_code: i32,
}

/// BPF network event structure (must match C definition)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct BpfNetworkEvent {
    event_type: u32,
    pid: u32,
    uid: u32,
    timestamp_ns: u64,
    comm: [u8; 256],
    family: u16,
    port: u16,
    addr_v4: u32,
    addr_v6: [u8; 16],
}

/// BPF file event structure (must match C definition)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct BpfFileEvent {
    event_type: u32,
    pid: u32,
    uid: u32,
    timestamp_ns: u64,
    comm: [u8; 256],
    path: [u8; 256],
    flags: i32,
    dirfd: i32,
}

/// Errors from eBPF monitor
#[derive(Error, Debug)]
pub enum EbpfError {
    #[error("eBPF not available on this system")]
    NotAvailable,
    #[error("Failed to load BPF program: {0}")]
    LoadFailed(String),
    #[error("Failed to attach BPF program: {0}")]
    AttachFailed(String),
    #[error("Ring buffer error: {0}")]
    RingBufferError(String),
    #[error("Insufficient privileges for eBPF")]
    InsufficientPrivileges,
}

/// eBPF-based process monitor
pub struct EbpfProcessMonitor {
    running: bool,
    processes: Arc<RwLock<HashMap<u32, ProcessInfo>>>,
    event_tx: broadcast::Sender<ProcessEvent>,
}

impl EbpfProcessMonitor {
    /// Create a new eBPF process monitor
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1024);

        Self {
            running: false,
            processes: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    /// Check if eBPF monitoring is available
    pub fn is_available() -> bool {
        // Check for CAP_BPF or root
        if !has_bpf_capability() {
            return false;
        }

        // Check for BTF support
        Path::new("/sys/kernel/btf/vmlinux").exists()
    }

    /// Start the eBPF monitor
    pub fn start(&mut self) -> Result<(), EbpfError> {
        if self.running {
            return Ok(());
        }

        if !Self::is_available() {
            return Err(EbpfError::NotAvailable);
        }

        info!("Starting eBPF process monitor");

        // Open and load BPF skeleton
        let skel_builder = ProcessMonitorSkelBuilder::default();
        let open_skel = skel_builder
            .open()
            .map_err(|e| EbpfError::LoadFailed(e.to_string()))?;

        let mut skel = open_skel
            .load()
            .map_err(|e| EbpfError::LoadFailed(e.to_string()))?;

        // Attach BPF programs
        skel.attach()
            .map_err(|e| EbpfError::AttachFailed(e.to_string()))?;

        info!("eBPF programs attached successfully");

        // Set up ring buffer polling
        let processes = self.processes.clone();
        let event_tx = self.event_tx.clone();

        // Spawn polling task
        std::thread::spawn(move || {
            Self::poll_events(skel, processes, event_tx);
        });

        self.running = true;
        Ok(())
    }

    /// Stop the eBPF monitor
    pub fn stop(&mut self) {
        self.running = false;
        info!("eBPF process monitor stopped");
    }

    /// Get a snapshot of currently tracked processes
    pub fn snapshot(&self) -> Vec<ProcessInfo> {
        self.processes.read().values().cloned().collect()
    }

    /// Subscribe to process events
    pub fn subscribe(&self) -> broadcast::Receiver<ProcessEvent> {
        self.event_tx.subscribe()
    }

    /// Poll events from BPF ring buffers (process, network, file)
    fn poll_events(
        mut skel: ProcessMonitorSkel<'static>,
        processes: Arc<RwLock<HashMap<u32, ProcessInfo>>>,
        event_tx: broadcast::Sender<ProcessEvent>,
    ) {
        // Process events callback
        let processes_clone = processes.clone();
        let event_tx_clone = event_tx.clone();
        let process_event_callback = move |data: &[u8]| {
            if data.len() < std::mem::size_of::<BpfProcessEvent>() {
                warn!("Received truncated BPF process event");
                return 0;
            }

            let event: &BpfProcessEvent = unsafe {
                &*(data.as_ptr() as *const BpfProcessEvent)
            };

            Self::handle_process_event(event, &processes_clone, &event_tx_clone);
            0
        };

        // Network events callback
        let network_event_callback = |data: &[u8]| {
            if data.len() < std::mem::size_of::<BpfNetworkEvent>() {
                warn!("Received truncated BPF network event");
                return 0;
            }

            let event: &BpfNetworkEvent = unsafe {
                &*(data.as_ptr() as *const BpfNetworkEvent)
            };

            Self::handle_network_event(event);
            0
        };

        // File events callback
        let file_event_callback = |data: &[u8]| {
            if data.len() < std::mem::size_of::<BpfFileEvent>() {
                warn!("Received truncated BPF file event");
                return 0;
            }

            let event: &BpfFileEvent = unsafe {
                &*(data.as_ptr() as *const BpfFileEvent)
            };

            Self::handle_file_event(event);
            0
        };

        // Build ring buffer with all event types
        let mut builder = RingBufferBuilder::new();
        builder
            .add(&skel.maps.events, process_event_callback)
            .expect("Failed to add process events ring buffer");
        builder
            .add(&skel.maps.network_events, network_event_callback)
            .expect("Failed to add network events ring buffer");
        builder
            .add(&skel.maps.file_events, file_event_callback)
            .expect("Failed to add file events ring buffer");

        let ring_buffer = builder.build().expect("Failed to build ring buffer");

        info!("eBPF ring buffers initialized for process, network, and file events");

        // Poll loop
        loop {
            if let Err(e) = ring_buffer.poll(Duration::from_millis(100)) {
                error!("Ring buffer poll error: {}", e);
            }
        }
    }

    /// Handle a network event from eBPF
    fn handle_network_event(event: &BpfNetworkEvent) {
        let comm = cstr_to_string(&event.comm);

        let addr_str = match event.family {
            AF_INET => {
                let bytes = event.addr_v4.to_be_bytes();
                format!("{}.{}.{}.{}:{}", bytes[0], bytes[1], bytes[2], bytes[3],
                        u16::from_be(event.port))
            }
            AF_INET6 => {
                format!("[{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}]:{}",
                    u16::from_be_bytes([event.addr_v6[0], event.addr_v6[1]]),
                    u16::from_be_bytes([event.addr_v6[2], event.addr_v6[3]]),
                    u16::from_be_bytes([event.addr_v6[4], event.addr_v6[5]]),
                    u16::from_be_bytes([event.addr_v6[6], event.addr_v6[7]]),
                    u16::from_be_bytes([event.addr_v6[8], event.addr_v6[9]]),
                    u16::from_be_bytes([event.addr_v6[10], event.addr_v6[11]]),
                    u16::from_be_bytes([event.addr_v6[12], event.addr_v6[13]]),
                    u16::from_be_bytes([event.addr_v6[14], event.addr_v6[15]]),
                    u16::from_be(event.port))
            }
            AF_UNIX => "unix".to_string(),
            _ => "unknown".to_string(),
        };

        debug!("BPF: Network connect: {} (PID: {}) -> {}", comm, event.pid, addr_str);
    }

    /// Handle a file event from eBPF
    fn handle_file_event(event: &BpfFileEvent) {
        let comm = cstr_to_string(&event.comm);
        let path = cstr_to_string(&event.path);

        // Filter out common noise paths
        if path.starts_with("/proc/") ||
           path.starts_with("/sys/") ||
           path.starts_with("/dev/") {
            return;
        }

        debug!("BPF: File open: {} (PID: {}) -> {} (flags: 0x{:x})",
               comm, event.pid, path, event.flags);
    }

    /// Handle a process event from eBPF
    fn handle_process_event(
        event: &BpfProcessEvent,
        processes: &Arc<RwLock<HashMap<u32, ProcessInfo>>>,
        event_tx: &broadcast::Sender<ProcessEvent>,
    ) {
        let comm = cstr_to_string(&event.comm);
        let filename = cstr_to_string(&event.filename);

        let timestamp = DateTime::from_timestamp_nanos(event.timestamp_ns as i64);

        match event.event_type {
            EVENT_PROCESS_EXEC => {
                let process = ProcessInfo {
                    id: Uuid::new_v4(),
                    pid: event.pid,
                    ppid: Some(event.ppid),
                    name: comm.clone(),
                    cmdline: None, // Would need additional BPF to capture
                    exe_path: Some(filename),
                    user: Some(event.uid.to_string()),
                    cwd: None,
                    start_time: timestamp,
                    end_time: None,
                    agent_type: None,
                };

                debug!("BPF: Process spawned: {} (PID: {})", comm, event.pid);

                {
                    let mut procs = processes.write();
                    procs.insert(event.pid, process.clone());
                }

                let _ = event_tx.send(ProcessEvent {
                    event_type: ProcessEventType::Spawn,
                    process,
                    timestamp,
                });
            }
            EVENT_PROCESS_EXIT => {
                debug!("BPF: Process exited: {} (PID: {}, code: {})", comm, event.pid, event.exit_code);

                let mut procs = processes.write();
                if let Some(mut process) = procs.remove(&event.pid) {
                    process.end_time = Some(timestamp);

                    let _ = event_tx.send(ProcessEvent {
                        event_type: ProcessEventType::Exit,
                        process,
                        timestamp,
                    });
                }
            }
            _ => {
                warn!("Unknown BPF event type: {}", event.event_type);
            }
        }
    }
}

impl Default for EbpfProcessMonitor {
    fn default() -> Self {
        Self::new()
    }
}

// Implement ProcessMonitorBackend trait for integration with ProcessMonitorService
impl super::ProcessMonitorBackend for EbpfProcessMonitor {
    fn start(&mut self) -> roea_common::PlatformResult<()> {
        self.start().map_err(|e| roea_common::PlatformError::Internal(e.to_string()))
    }

    fn stop(&mut self) -> roea_common::PlatformResult<()> {
        self.stop();
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn snapshot(&self) -> roea_common::PlatformResult<Vec<roea_common::ProcessInfo>> {
        Ok(self.snapshot())
    }
}

/// Convert C string bytes to Rust String
fn cstr_to_string(bytes: &[u8]) -> String {
    let nul_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..nul_pos]).to_string()
}

/// Check if we have BPF capability
fn has_bpf_capability() -> bool {
    #[cfg(target_os = "linux")]
    {
        // Check if running as root
        unsafe {
            if libc::geteuid() == 0 {
                return true;
            }
        }

        // Could also check for CAP_BPF capability here
        // For now, just check root
        false
    }

    #[cfg(not(target_os = "linux"))]
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cstr_to_string() {
        let bytes = b"hello\0world";
        assert_eq!(cstr_to_string(bytes), "hello");

        let bytes = b"no null terminator";
        assert_eq!(cstr_to_string(bytes), "no null terminator");
    }

    #[test]
    fn test_monitor_creation() {
        let monitor = EbpfProcessMonitor::new();
        assert!(!monitor.running);
    }
}
