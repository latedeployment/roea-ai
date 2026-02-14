//! File access monitoring
//!
//! Tracks file operations by AI agent processes.

mod proc_fd;

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use parking_lot::RwLock;
use tuai_common::{FileOpInfo, PlatformResult};
use tokio::sync::broadcast;

pub use proc_fd::ProcFdMonitor;

/// Paths to filter out from file monitoring (noise reduction)
const NOISE_PATTERNS: &[&str] = &[
    "/proc/",
    "/sys/",
    "/dev/",
    "/run/",
    "/tmp/",
    "node_modules/",
    ".git/objects/",
    "__pycache__/",
    ".cache/",
    ".npm/",
    ".cargo/registry/",
];

/// File monitor service
pub struct FileMonitorService {
    inner: Box<dyn FileMonitorBackend>,
    event_tx: broadcast::Sender<FileOpInfo>,
    /// Paths being watched
    watched_paths: RwLock<HashSet<String>>,
    /// Whether to filter noise
    filter_noise: bool,
}

impl FileMonitorService {
    /// Create a new file monitor service
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        Self {
            inner: Box::new(ProcFdMonitor::new()),
            event_tx,
            watched_paths: RwLock::new(HashSet::new()),
            filter_noise: true,
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

    /// Get current open files for a process
    pub fn open_files_for_pid(&self, pid: u32) -> PlatformResult<Vec<FileOpInfo>> {
        let files = self.inner.open_files_for_pid(pid)?;

        if self.filter_noise {
            Ok(files
                .into_iter()
                .filter(|f| !Self::is_noise_path(&f.path))
                .collect())
        } else {
            Ok(files)
        }
    }

    /// Get all open files across all processes
    pub fn snapshot(&self) -> PlatformResult<Vec<FileOpInfo>> {
        let files = self.inner.snapshot()?;

        if self.filter_noise {
            Ok(files
                .into_iter()
                .filter(|f| !Self::is_noise_path(&f.path))
                .collect())
        } else {
            Ok(files)
        }
    }

    /// Add a path to watch
    pub fn watch_path(&mut self, path: &str) -> PlatformResult<()> {
        self.watched_paths.write().insert(path.to_string());
        Ok(())
    }

    /// Remove a path from watching
    pub fn unwatch_path(&mut self, path: &str) -> PlatformResult<()> {
        self.watched_paths.write().remove(path);
        Ok(())
    }

    /// Subscribe to file events
    pub fn subscribe(&self) -> broadcast::Receiver<FileOpInfo> {
        self.event_tx.subscribe()
    }

    /// Check if a path is noise that should be filtered
    pub(crate) fn is_noise_path(path: &str) -> bool {
        NOISE_PATTERNS.iter().any(|pattern| path.contains(pattern))
    }

    /// Classify a file path
    pub fn classify_path(path: &str) -> FileCategory {
        if path.ends_with(".rs")
            || path.ends_with(".py")
            || path.ends_with(".js")
            || path.ends_with(".ts")
            || path.ends_with(".go")
            || path.ends_with(".java")
            || path.ends_with(".c")
            || path.ends_with(".cpp")
            || path.ends_with(".h")
        {
            FileCategory::SourceCode
        } else if path.ends_with(".json")
            || path.ends_with(".yaml")
            || path.ends_with(".yml")
            || path.ends_with(".toml")
            || path.ends_with(".ini")
            || path.ends_with(".env")
        {
            FileCategory::Config
        } else if path.ends_with(".md") || path.ends_with(".txt") || path.ends_with(".rst") {
            FileCategory::Documentation
        } else if path.contains(".git/") {
            FileCategory::Git
        } else if path.ends_with(".lock")
            || path.contains("package-lock")
            || path.contains("Cargo.lock")
        {
            FileCategory::LockFile
        } else if path.contains("node_modules/")
            || path.contains("target/")
            || path.contains("__pycache__/")
        {
            FileCategory::BuildArtifact
        } else {
            FileCategory::Other
        }
    }
}

impl Default for FileMonitorService {
    fn default() -> Self {
        Self::new()
    }
}

/// Classification of file types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileCategory {
    /// Source code files
    SourceCode,
    /// Configuration files
    Config,
    /// Documentation
    Documentation,
    /// Git-related files
    Git,
    /// Lock files
    LockFile,
    /// Build artifacts
    BuildArtifact,
    /// Other files
    Other,
}

/// Trait for file monitoring backends
pub trait FileMonitorBackend: Send + Sync {
    fn start(&mut self) -> PlatformResult<()>;
    fn stop(&mut self) -> PlatformResult<()>;
    fn is_running(&self) -> bool;
    fn snapshot(&self) -> PlatformResult<Vec<FileOpInfo>>;
    fn open_files_for_pid(&self, pid: u32) -> PlatformResult<Vec<FileOpInfo>>;
}
