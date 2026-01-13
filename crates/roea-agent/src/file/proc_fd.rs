//! Linux /proc/*/fd based file monitor
//!
//! Tracks open file descriptors for processes.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::Utc;
use roea_common::{FileOpInfo, FileOperation, PlatformError, PlatformResult};

use super::FileMonitorBackend;

/// File monitor using /proc/*/fd (Linux)
pub struct ProcFdMonitor {
    running: AtomicBool,
}

impl ProcFdMonitor {
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
        }
    }

    /// Get open files for a specific PID
    fn read_fd_for_pid(pid: u32) -> PlatformResult<Vec<FileOpInfo>> {
        let fd_path = format!("/proc/{}/fd", pid);
        let fd_dir = Path::new(&fd_path);

        if !fd_dir.exists() {
            return Ok(vec![]);
        }

        let mut files = Vec::new();

        let entries = match fs::read_dir(fd_dir) {
            Ok(e) => e,
            Err(_) => return Ok(vec![]), // Process may have exited or permission denied
        };

        for entry in entries.flatten() {
            // Read the symlink to get the actual file path
            if let Ok(link_target) = fs::read_link(entry.path()) {
                let path_str = link_target.to_string_lossy().to_string();

                // Skip non-file entries (sockets, pipes, etc.)
                if path_str.starts_with("socket:")
                    || path_str.starts_with("pipe:")
                    || path_str.starts_with("anon_inode:")
                    || path_str.starts_with("/dev/")
                    || path_str.starts_with("/proc/")
                    || path_str.starts_with("/sys/")
                {
                    continue;
                }

                // Try to determine operation type from fdinfo
                let operation = Self::get_fd_operation(pid, &entry.file_name());

                let file_op = FileOpInfo::new(pid, operation, path_str);
                files.push(file_op);
            }
        }

        Ok(files)
    }

    /// Try to determine the operation type from /proc/*/fdinfo/*
    fn get_fd_operation(pid: u32, fd: &std::ffi::OsStr) -> FileOperation {
        let fdinfo_path = format!("/proc/{}/fdinfo/{}", pid, fd.to_string_lossy());

        if let Ok(content) = fs::read_to_string(&fdinfo_path) {
            // Parse flags to determine read/write mode
            for line in content.lines() {
                if line.starts_with("flags:") {
                    let flags_str = line.trim_start_matches("flags:").trim();
                    if let Ok(flags) = u32::from_str_radix(flags_str.trim_start_matches("0"), 8) {
                        // O_RDONLY = 0, O_WRONLY = 1, O_RDWR = 2
                        let access_mode = flags & 3;
                        return match access_mode {
                            0 => FileOperation::Read,
                            1 => FileOperation::Write,
                            2 => FileOperation::Write, // RDWR counts as write
                            _ => FileOperation::Open,
                        };
                    }
                }
            }
        }

        FileOperation::Open
    }
}

impl Default for ProcFdMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl FileMonitorBackend for ProcFdMonitor {
    fn start(&mut self) -> PlatformResult<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        #[cfg(not(target_os = "linux"))]
        {
            return Err(PlatformError::NotSupported(
                "/proc/*/fd is only available on Linux".to_string(),
            ));
        }

        self.running.store(true, Ordering::Relaxed);
        tracing::info!("ProcFd file monitor started");
        Ok(())
    }

    fn stop(&mut self) -> PlatformResult<()> {
        self.running.store(false, Ordering::Relaxed);
        tracing::info!("ProcFd file monitor stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn snapshot(&self) -> PlatformResult<Vec<FileOpInfo>> {
        let mut all_files = Vec::new();

        let proc_dir = match fs::read_dir("/proc") {
            Ok(dir) => dir,
            Err(e) => {
                return Err(PlatformError::CollectionFailed(format!(
                    "Failed to read /proc: {}",
                    e
                )))
            }
        };

        for entry in proc_dir.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Only process numeric directories (PIDs)
            if !name_str.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            if let Ok(pid) = name_str.parse::<u32>() {
                if let Ok(files) = Self::read_fd_for_pid(pid) {
                    all_files.extend(files);
                }
            }
        }

        Ok(all_files)
    }

    fn open_files_for_pid(&self, pid: u32) -> PlatformResult<Vec<FileOpInfo>> {
        Self::read_fd_for_pid(pid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_process_files() {
        let monitor = ProcFdMonitor::new();
        let pid = std::process::id();
        let files = monitor.open_files_for_pid(pid);
        assert!(files.is_ok());
        // Current process should have at least stdout/stderr open
        // but those are filtered as /dev/
    }
}
