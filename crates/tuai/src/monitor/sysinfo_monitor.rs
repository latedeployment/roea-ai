//! Sysinfo-based process monitor (cross-platform fallback)
//!
//! Uses the sysinfo crate to poll process information.
//! This is a fallback when platform-specific APIs are unavailable.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::Utc;
use parking_lot::RwLock;
use tuai_common::{PlatformResult, ProcessInfo};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use super::ProcessMonitorBackend;

/// Process monitor using sysinfo crate
pub struct SysinfoMonitor {
    system: Arc<RwLock<System>>,
    running: AtomicBool,
    /// Cache of known processes with their internal IDs
    process_cache: Arc<RwLock<HashMap<u32, ProcessInfo>>>,
}

impl SysinfoMonitor {
    /// Create a new sysinfo-based monitor
    pub fn new() -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new())),
            running: AtomicBool::new(false),
            process_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Refresh process list and detect changes
    fn refresh(&self) -> PlatformResult<()> {
        let mut system = self.system.write();
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cmd(UpdateKind::Always)
                .with_cwd(UpdateKind::Always)
                .with_exe(UpdateKind::Always)
                .with_user(UpdateKind::Always),
        );
        Ok(())
    }

    /// Convert sysinfo process to our ProcessInfo
    fn convert_process(pid: Pid, proc: &sysinfo::Process) -> ProcessInfo {
        let mut info = ProcessInfo::new(pid.as_u32(), proc.name().to_string_lossy().to_string());

        info.ppid = proc.parent().map(|p| p.as_u32());
        // cmd() returns &[OsString], so we need to convert each to string and join
        let cmd_parts: Vec<String> = proc.cmd().iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect();
        info.cmdline = Some(cmd_parts.join(" "));
        info.exe_path = proc.exe().map(|p| p.to_string_lossy().to_string());
        info.cwd = proc.cwd().map(|p| p.to_string_lossy().to_string());
        // Uid doesn't implement Display, use Debug format
        info.user = proc.user_id().map(|u| format!("{:?}", u));
        info.start_time = chrono::DateTime::from_timestamp(proc.start_time() as i64, 0)
            .unwrap_or_else(Utc::now);

        info
    }
}

impl Default for SysinfoMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessMonitorBackend for SysinfoMonitor {
    fn start(&mut self) -> PlatformResult<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        // Initial refresh
        self.refresh()?;

        self.running.store(true, Ordering::Relaxed);
        tracing::info!("Sysinfo process monitor started");
        Ok(())
    }

    fn stop(&mut self) -> PlatformResult<()> {
        self.running.store(false, Ordering::Relaxed);
        tracing::info!("Sysinfo process monitor stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn snapshot(&self) -> PlatformResult<Vec<ProcessInfo>> {
        // Refresh before taking snapshot
        self.refresh()?;

        let system = self.system.read();
        let processes: Vec<ProcessInfo> = system
            .processes()
            .iter()
            .map(|(pid, proc)| Self::convert_process(*pid, proc))
            .collect();

        Ok(processes)
    }
}
