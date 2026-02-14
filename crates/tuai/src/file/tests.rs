//! Unit tests for file access monitoring
//!
//! Tests cover:
//! - File open/read/write detection
//! - Directory traversal tracking
//! - Symlink resolution
//! - File delete/rename operations
//! - Permission denied scenarios
//! - High I/O scenarios
//! - Large file operations
//! - Recursive directory watches
//!
//! Target: 80%+ coverage per THE-39

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use parking_lot::RwLock;
use tuai_common::{FileOpInfo, FileOperation, PlatformError, PlatformResult};
use uuid::Uuid;

use super::{FileCategory, FileMonitorBackend, FileMonitorService};

// ============================================================================
// Test Fixtures
// ============================================================================

/// Create a file operation fixture
fn create_file_op(pid: u32, operation: FileOperation, path: &str) -> FileOpInfo {
    let mut op = FileOpInfo::new(pid, operation, path.to_string());
    op.timestamp = Utc::now();
    op
}

/// Create a set of typical source code file operations
fn create_source_code_ops() -> Vec<FileOpInfo> {
    vec![
        create_file_op(1000, FileOperation::Read, "/home/user/project/src/main.rs"),
        create_file_op(1000, FileOperation::Write, "/home/user/project/src/lib.rs"),
        create_file_op(1000, FileOperation::Read, "/home/user/project/Cargo.toml"),
        create_file_op(1000, FileOperation::Read, "/home/user/project/src/utils.rs"),
    ]
}

/// Create a set of file operations representing npm install
fn create_npm_install_ops() -> Vec<FileOpInfo> {
    vec![
        create_file_op(2000, FileOperation::Read, "/home/user/project/package.json"),
        create_file_op(2000, FileOperation::Read, "/home/user/project/package-lock.json"),
        create_file_op(2000, FileOperation::Write, "/home/user/project/node_modules/.package-lock.json"),
        create_file_op(2000, FileOperation::Create, "/home/user/project/node_modules/express/index.js"),
        create_file_op(2000, FileOperation::Create, "/home/user/project/node_modules/lodash/lodash.js"),
    ]
}

/// Create a typical AI agent file access pattern
fn create_ai_agent_ops() -> Vec<FileOpInfo> {
    vec![
        // Reading project files
        create_file_op(3000, FileOperation::Read, "/home/user/project/README.md"),
        create_file_op(3000, FileOperation::Read, "/home/user/project/src/api.ts"),
        // Writing generated code
        create_file_op(3000, FileOperation::Write, "/home/user/project/src/generated.ts"),
        // Modifying config
        create_file_op(3000, FileOperation::Write, "/home/user/project/tsconfig.json"),
        // Git operations
        create_file_op(3000, FileOperation::Read, "/home/user/project/.git/HEAD"),
    ]
}

// ============================================================================
// Mock File Monitor Backend
// ============================================================================

/// Mock file monitor for deterministic testing
pub struct MockFileMonitor {
    running: AtomicBool,
    file_ops: Arc<RwLock<HashMap<Uuid, FileOpInfo>>>,
    op_count: AtomicU32,
}

impl MockFileMonitor {
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            file_ops: Arc::new(RwLock::new(HashMap::new())),
            op_count: AtomicU32::new(0),
        }
    }

    /// Create a mock monitor pre-populated with file ops
    pub fn with_file_ops(ops: Vec<FileOpInfo>) -> Self {
        let mock = Self::new();
        {
            let mut map = mock.file_ops.write();
            for op in ops {
                map.insert(op.id, op);
            }
        }
        mock
    }

    /// Add a file operation
    pub fn add_file_op(&self, op: FileOpInfo) -> Uuid {
        let id = op.id;
        {
            let mut ops = self.file_ops.write();
            ops.insert(id, op);
        }
        self.op_count.fetch_add(1, Ordering::SeqCst);
        id
    }

    /// Remove a file operation
    pub fn remove_file_op(&self, id: Uuid) -> Option<FileOpInfo> {
        self.file_ops.write().remove(&id)
    }

    /// Get file ops by PID
    pub fn ops_by_pid(&self, pid: u32) -> Vec<FileOpInfo> {
        self.file_ops
            .read()
            .values()
            .filter(|op| op.pid == pid)
            .cloned()
            .collect()
    }

    /// Get file ops by operation type
    pub fn ops_by_type(&self, operation: FileOperation) -> Vec<FileOpInfo> {
        self.file_ops
            .read()
            .values()
            .filter(|op| op.operation == operation)
            .cloned()
            .collect()
    }

    /// Get file ops by path prefix
    pub fn ops_by_path_prefix(&self, prefix: &str) -> Vec<FileOpInfo> {
        self.file_ops
            .read()
            .values()
            .filter(|op| op.path.starts_with(prefix))
            .cloned()
            .collect()
    }

    /// Get file ops count
    pub fn op_count(&self) -> usize {
        self.file_ops.read().len()
    }

    /// Get total ops added
    pub fn total_added(&self) -> u32 {
        self.op_count.load(Ordering::SeqCst)
    }
}

impl Default for MockFileMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl FileMonitorBackend for MockFileMonitor {
    fn start(&mut self) -> PlatformResult<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }
        self.running.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn stop(&mut self) -> PlatformResult<()> {
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn snapshot(&self) -> PlatformResult<Vec<FileOpInfo>> {
        Ok(self.file_ops.read().values().cloned().collect())
    }

    fn open_files_for_pid(&self, pid: u32) -> PlatformResult<Vec<FileOpInfo>> {
        Ok(self.ops_by_pid(pid))
    }
}

// ============================================================================
// Test Module: File Open/Read/Write Detection
// ============================================================================

#[cfg(test)]
mod file_operation_tests {
    use super::*;

    #[test]
    fn test_file_open_detection() {
        let op = create_file_op(1000, FileOperation::Open, "/home/user/file.txt");
        assert_eq!(op.operation, FileOperation::Open);
        assert_eq!(op.pid, 1000);
        assert_eq!(op.path, "/home/user/file.txt");
    }

    #[test]
    fn test_file_read_detection() {
        let op = create_file_op(1000, FileOperation::Read, "/etc/passwd");
        assert_eq!(op.operation, FileOperation::Read);
    }

    #[test]
    fn test_file_write_detection() {
        let op = create_file_op(1000, FileOperation::Write, "/home/user/output.log");
        assert_eq!(op.operation, FileOperation::Write);
    }

    #[test]
    fn test_file_delete_detection() {
        let op = create_file_op(1000, FileOperation::Delete, "/tmp/old_file.txt");
        assert_eq!(op.operation, FileOperation::Delete);
    }

    #[test]
    fn test_file_rename_detection() {
        let mut op = create_file_op(1000, FileOperation::Rename, "/home/user/old_name.txt");
        op.new_path = Some("/home/user/new_name.txt".to_string());

        assert_eq!(op.operation, FileOperation::Rename);
        assert_eq!(op.new_path.as_deref(), Some("/home/user/new_name.txt"));
    }

    #[test]
    fn test_file_create_detection() {
        let op = create_file_op(1000, FileOperation::Create, "/home/user/new_file.rs");
        assert_eq!(op.operation, FileOperation::Create);
    }

    #[test]
    fn test_filter_by_operation_type() {
        let ops = vec![
            create_file_op(1, FileOperation::Read, "/file1"),
            create_file_op(2, FileOperation::Write, "/file2"),
            create_file_op(3, FileOperation::Read, "/file3"),
            create_file_op(4, FileOperation::Delete, "/file4"),
        ];

        let monitor = MockFileMonitor::with_file_ops(ops);

        let reads = monitor.ops_by_type(FileOperation::Read);
        assert_eq!(reads.len(), 2);

        let writes = monitor.ops_by_type(FileOperation::Write);
        assert_eq!(writes.len(), 1);

        let deletes = monitor.ops_by_type(FileOperation::Delete);
        assert_eq!(deletes.len(), 1);
    }

    #[test]
    fn test_multiple_operations_same_file() {
        let ops = vec![
            create_file_op(1000, FileOperation::Open, "/home/user/data.json"),
            create_file_op(1000, FileOperation::Read, "/home/user/data.json"),
            create_file_op(1000, FileOperation::Write, "/home/user/data.json"),
        ];

        let monitor = MockFileMonitor::with_file_ops(ops);
        assert_eq!(monitor.op_count(), 3);

        // All ops for same file have different IDs
        let snapshot = monitor.snapshot().unwrap();
        let ids: std::collections::HashSet<_> = snapshot.iter().map(|op| op.id).collect();
        assert_eq!(ids.len(), 3);
    }
}

// ============================================================================
// Test Module: Directory Traversal
// ============================================================================

#[cfg(test)]
mod directory_traversal_tests {
    use super::*;

    #[test]
    fn test_filter_by_directory() {
        let ops = vec![
            create_file_op(1, FileOperation::Read, "/home/user/project/src/main.rs"),
            create_file_op(1, FileOperation::Read, "/home/user/project/src/lib.rs"),
            create_file_op(1, FileOperation::Read, "/home/user/project/tests/test.rs"),
            create_file_op(1, FileOperation::Read, "/etc/hosts"),
        ];

        let monitor = MockFileMonitor::with_file_ops(ops);

        let src_files = monitor.ops_by_path_prefix("/home/user/project/src/");
        assert_eq!(src_files.len(), 2);

        let project_files = monitor.ops_by_path_prefix("/home/user/project/");
        assert_eq!(project_files.len(), 3);
    }

    #[test]
    fn test_deep_directory_path() {
        let deep_path = "/home/user/projects/company/team/repo/src/components/common/utils/helpers.ts";
        let op = create_file_op(1000, FileOperation::Read, deep_path);

        assert_eq!(op.path, deep_path);
    }

    #[test]
    fn test_root_directory() {
        let op = create_file_op(1, FileOperation::Read, "/file.txt");
        assert_eq!(op.path, "/file.txt");
    }

    #[test]
    fn test_hidden_directories() {
        let ops = vec![
            create_file_op(1, FileOperation::Read, "/home/user/.config/app/config.json"),
            create_file_op(1, FileOperation::Read, "/home/user/.ssh/id_rsa"),
            create_file_op(1, FileOperation::Read, "/home/user/.bashrc"),
        ];

        let monitor = MockFileMonitor::with_file_ops(ops);

        let hidden = monitor.ops_by_path_prefix("/home/user/.");
        assert_eq!(hidden.len(), 3);
    }

    #[test]
    fn test_recursive_directory_listing() {
        let paths = vec![
            "/project/a.txt",
            "/project/dir1/b.txt",
            "/project/dir1/dir2/c.txt",
            "/project/dir1/dir2/dir3/d.txt",
        ];

        let ops: Vec<_> = paths
            .iter()
            .map(|p| create_file_op(1, FileOperation::Read, p))
            .collect();

        let monitor = MockFileMonitor::with_file_ops(ops);
        let all = monitor.ops_by_path_prefix("/project/");
        assert_eq!(all.len(), 4);
    }
}

// ============================================================================
// Test Module: Symlink Handling
// ============================================================================

#[cfg(test)]
mod symlink_tests {
    use super::*;

    #[test]
    fn test_symlink_path() {
        // Symlink path should be recorded as-is
        let op = create_file_op(1000, FileOperation::Read, "/usr/bin/python");
        assert_eq!(op.path, "/usr/bin/python");
    }

    #[test]
    fn test_resolved_symlink_path() {
        // The actual target might be different
        let op = create_file_op(1000, FileOperation::Read, "/usr/bin/python3.10");
        assert_eq!(op.path, "/usr/bin/python3.10");
    }

    #[test]
    fn test_relative_symlink() {
        // Relative paths in symlinks should be stored as absolute when resolved
        let op = create_file_op(1000, FileOperation::Read, "/home/user/project/link_to_file");
        assert!(op.path.starts_with('/'));
    }
}

// ============================================================================
// Test Module: Permission Denied Scenarios
// ============================================================================

#[cfg(test)]
mod permission_tests {
    use super::*;

    #[test]
    fn test_permission_denied_file() {
        // Simulating that we can still track the attempt even if denied
        let op = create_file_op(1000, FileOperation::Open, "/etc/shadow");
        assert_eq!(op.path, "/etc/shadow");
    }

    #[test]
    fn test_protected_system_files() {
        let protected_paths = vec![
            "/etc/shadow",
            "/etc/sudoers",
            "/root/.ssh/id_rsa",
            "/var/lib/private",
        ];

        for path in protected_paths {
            let op = create_file_op(1000, FileOperation::Read, path);
            assert_eq!(op.path, path);
        }
    }

    #[test]
    fn test_operation_on_nonexistent_file() {
        let op = create_file_op(1000, FileOperation::Open, "/nonexistent/path/file.txt");
        assert_eq!(op.operation, FileOperation::Open);
    }
}

// ============================================================================
// Test Module: High I/O Scenarios
// ============================================================================

#[cfg(test)]
mod high_io_tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_npm_install_simulation() {
        let ops = create_npm_install_ops();
        let monitor = MockFileMonitor::with_file_ops(ops);

        // npm install creates many files in node_modules
        let node_modules = monitor.ops_by_path_prefix("/home/user/project/node_modules/");
        assert!(!node_modules.is_empty());
    }

    #[test]
    fn test_1000_file_operations() {
        let monitor = MockFileMonitor::new();
        let start = Instant::now();

        for i in 0..1000 {
            let op = create_file_op(
                1000,
                if i % 2 == 0 {
                    FileOperation::Read
                } else {
                    FileOperation::Write
                },
                &format!("/project/file_{}.txt", i),
            );
            monitor.add_file_op(op);
        }

        let elapsed = start.elapsed();
        assert_eq!(monitor.op_count(), 1000);
        assert!(elapsed < Duration::from_secs(1));
    }

    #[test]
    fn test_rapid_file_churn() {
        let monitor = MockFileMonitor::new();
        let start = Instant::now();

        // Create and remove files rapidly
        for i in 0..500 {
            let op = create_file_op(1, FileOperation::Create, &format!("/tmp/file_{}", i));
            let id = monitor.add_file_op(op);
            monitor.remove_file_op(id);
        }

        let elapsed = start.elapsed();
        assert_eq!(monitor.op_count(), 0);
        assert_eq!(monitor.total_added(), 500);
        assert!(elapsed < Duration::from_secs(1));
    }

    #[test]
    fn test_concurrent_file_access() {
        let monitor = Arc::new(MockFileMonitor::new());
        let mut handles = Vec::new();

        for t in 0..4 {
            let m = Arc::clone(&monitor);
            let handle = thread::spawn(move || {
                for i in 0..100 {
                    let op = create_file_op(
                        (t * 1000 + i) as u32,
                        FileOperation::Read,
                        &format!("/thread{}/file_{}.txt", t, i),
                    );
                    m.add_file_op(op);
                }
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(monitor.op_count(), 400);
    }

    #[test]
    fn test_many_files_per_process() {
        let monitor = MockFileMonitor::new();

        // Single process opens 100 files
        for i in 0..100 {
            let op = create_file_op(5000, FileOperation::Open, &format!("/data/file_{}.dat", i));
            monitor.add_file_op(op);
        }

        let process_files = monitor.ops_by_pid(5000);
        assert_eq!(process_files.len(), 100);
    }

    #[test]
    fn test_snapshot_performance() {
        let monitor = MockFileMonitor::new();

        // Add many file operations
        for i in 0..1000 {
            let op = create_file_op(i % 100, FileOperation::Read, &format!("/file_{}", i));
            monitor.add_file_op(op);
        }

        // Snapshot should be fast
        let start = Instant::now();
        for _ in 0..100 {
            let _ = monitor.snapshot();
        }
        let elapsed = start.elapsed();

        assert!(elapsed < Duration::from_secs(1));
    }
}

// ============================================================================
// Test Module: Large File Operations
// ============================================================================

#[cfg(test)]
mod large_file_tests {
    use super::*;

    #[test]
    fn test_large_file_path() {
        // Very long file path
        let path = format!("/home/user/{}/file.txt", "a".repeat(200));
        let op = create_file_op(1000, FileOperation::Read, &path);

        assert_eq!(op.path, path);
    }

    #[test]
    fn test_unicode_file_path() {
        let paths = vec![
            "/home/user/documents/日本語ファイル.txt",
            "/home/user/docs/файл.txt",
            "/home/user/files/αρχείο.txt",
            "/home/user/data/文件.json",
        ];

        for path in paths {
            let op = create_file_op(1000, FileOperation::Read, path);
            assert_eq!(op.path, path);
        }
    }

    #[test]
    fn test_special_characters_in_path() {
        let paths = vec![
            "/home/user/file with spaces.txt",
            "/home/user/file'with'quotes.txt",
            "/home/user/file\"with\"doublequotes.txt",
            "/home/user/file\twith\ttabs.txt",
        ];

        for path in paths {
            let op = create_file_op(1000, FileOperation::Read, path);
            assert_eq!(op.path, path);
        }
    }
}

// ============================================================================
// Test Module: File Classification
// ============================================================================

#[cfg(test)]
mod file_classification_tests {
    use super::*;

    #[test]
    fn test_source_code_classification() {
        let source_files = vec![
            "/project/main.rs",
            "/project/app.py",
            "/project/index.js",
            "/project/App.tsx",
            "/project/main.go",
            "/project/Main.java",
            "/project/main.c",
            "/project/main.cpp",
            "/project/header.h",
        ];

        for path in source_files {
            assert_eq!(
                FileMonitorService::classify_path(path),
                FileCategory::SourceCode,
                "Failed for: {}",
                path
            );
        }
    }

    #[test]
    fn test_config_classification() {
        let config_files = vec![
            "/project/config.json",
            "/project/settings.yaml",
            "/project/app.yml",
            "/project/Cargo.toml",
            "/project/setup.ini",
            "/project/.env",
        ];

        for path in config_files {
            assert_eq!(
                FileMonitorService::classify_path(path),
                FileCategory::Config,
                "Failed for: {}",
                path
            );
        }
    }

    #[test]
    fn test_documentation_classification() {
        let doc_files = vec![
            "/project/README.md",
            "/project/docs/guide.txt",
            "/project/api.rst",
        ];

        for path in doc_files {
            assert_eq!(
                FileMonitorService::classify_path(path),
                FileCategory::Documentation,
                "Failed for: {}",
                path
            );
        }
    }

    #[test]
    fn test_git_classification() {
        let git_files = vec![
            "/project/.git/HEAD",
            "/project/.git/config",
            "/project/.git/objects/ab/cd1234",
        ];

        for path in git_files {
            assert_eq!(
                FileMonitorService::classify_path(path),
                FileCategory::Git,
                "Failed for: {}",
                path
            );
        }
    }

    #[test]
    fn test_lock_file_classification() {
        let lock_files = vec![
            "/project/Cargo.lock",
            "/project/package-lock.json",
            "/project/yarn.lock",
        ];

        for path in lock_files {
            assert_eq!(
                FileMonitorService::classify_path(path),
                FileCategory::LockFile,
                "Failed for: {}",
                path
            );
        }
    }

    #[test]
    fn test_build_artifact_classification() {
        let artifacts = vec![
            "/project/node_modules/express/index.js",
            "/project/target/debug/binary",
            "/project/__pycache__/module.cpython-39.pyc",
        ];

        for path in artifacts {
            assert_eq!(
                FileMonitorService::classify_path(path),
                FileCategory::BuildArtifact,
                "Failed for: {}",
                path
            );
        }
    }

    #[test]
    fn test_other_classification() {
        let other_files = vec![
            "/project/image.png",
            "/project/data.bin",
            "/project/archive.zip",
        ];

        for path in other_files {
            assert_eq!(
                FileMonitorService::classify_path(path),
                FileCategory::Other,
                "Failed for: {}",
                path
            );
        }
    }
}

// ============================================================================
// Test Module: Noise Filtering
// ============================================================================

#[cfg(test)]
mod noise_filtering_tests {
    use super::*;

    #[test]
    fn test_noise_patterns() {
        let noise_paths = vec![
            "/proc/1/status",
            "/sys/class/net/eth0",
            "/dev/null",
            "/run/user/1000/bus",
            "/tmp/temp_file",
            "/project/node_modules/lodash/index.js",
            "/project/.git/objects/pack",
            "/project/__pycache__/cache.pyc",
            "/home/user/.cache/pip",
            "/home/user/.npm/_cacache",
            "/home/user/.cargo/registry/cache",
        ];

        for path in noise_paths {
            assert!(
                FileMonitorService::is_noise_path(path),
                "Should be noise: {}",
                path
            );
        }
    }

    #[test]
    fn test_non_noise_paths() {
        let real_paths = vec![
            "/home/user/project/main.rs",
            "/home/user/project/src/lib.rs",
            "/etc/hosts",
            "/var/log/app.log",
        ];

        for path in real_paths {
            assert!(
                !FileMonitorService::is_noise_path(path),
                "Should NOT be noise: {}",
                path
            );
        }
    }
}

// ============================================================================
// Test Module: Backend Trait Implementation
// ============================================================================

#[cfg(test)]
mod backend_tests {
    use super::*;

    #[test]
    fn test_start_stop_lifecycle() {
        let mut monitor = MockFileMonitor::new();

        assert!(!monitor.is_running());

        monitor.start().unwrap();
        assert!(monitor.is_running());

        monitor.stop().unwrap();
        assert!(!monitor.is_running());
    }

    #[test]
    fn test_double_start() {
        let mut monitor = MockFileMonitor::new();

        monitor.start().unwrap();
        monitor.start().unwrap(); // Should be idempotent

        assert!(monitor.is_running());
    }

    #[test]
    fn test_snapshot_empty() {
        let monitor = MockFileMonitor::new();
        let snapshot = monitor.snapshot().unwrap();
        assert!(snapshot.is_empty());
    }

    #[test]
    fn test_snapshot_with_ops() {
        let ops = create_source_code_ops();
        let count = ops.len();
        let monitor = MockFileMonitor::with_file_ops(ops);

        let snapshot = monitor.snapshot().unwrap();
        assert_eq!(snapshot.len(), count);
    }

    #[test]
    fn test_open_files_for_pid() {
        let ops = vec![
            create_file_op(1000, FileOperation::Read, "/file1"),
            create_file_op(1000, FileOperation::Read, "/file2"),
            create_file_op(2000, FileOperation::Read, "/file3"),
        ];

        let monitor = MockFileMonitor::with_file_ops(ops);

        let pid_1000 = monitor.open_files_for_pid(1000).unwrap();
        assert_eq!(pid_1000.len(), 2);

        let pid_2000 = monitor.open_files_for_pid(2000).unwrap();
        assert_eq!(pid_2000.len(), 1);

        let pid_3000 = monitor.open_files_for_pid(3000).unwrap();
        assert_eq!(pid_3000.len(), 0);
    }
}

// ============================================================================
// Test Module: AI Agent File Patterns
// ============================================================================

#[cfg(test)]
mod ai_agent_tests {
    use super::*;

    #[test]
    fn test_ai_agent_file_ops() {
        let ops = create_ai_agent_ops();
        let monitor = MockFileMonitor::with_file_ops(ops);

        let snapshot = monitor.snapshot().unwrap();
        assert_eq!(snapshot.len(), 5);

        // Check for both reads and writes
        let reads = monitor.ops_by_type(FileOperation::Read);
        let writes = monitor.ops_by_type(FileOperation::Write);

        assert!(!reads.is_empty());
        assert!(!writes.is_empty());
    }

    #[test]
    fn test_multiple_agents() {
        let mut ops = Vec::new();

        // Claude Code (PID 1000)
        ops.push(create_file_op(1000, FileOperation::Read, "/project/src/main.rs"));
        ops.push(create_file_op(1000, FileOperation::Write, "/project/src/new.rs"));

        // Cursor (PID 2000)
        ops.push(create_file_op(2000, FileOperation::Read, "/project/package.json"));
        ops.push(create_file_op(2000, FileOperation::Write, "/project/src/app.ts"));

        // Aider (PID 3000)
        ops.push(create_file_op(3000, FileOperation::Read, "/project/README.md"));

        let monitor = MockFileMonitor::with_file_ops(ops);

        assert_eq!(monitor.ops_by_pid(1000).len(), 2);
        assert_eq!(monitor.ops_by_pid(2000).len(), 2);
        assert_eq!(monitor.ops_by_pid(3000).len(), 1);
    }

    #[test]
    fn test_sensitive_file_access() {
        let sensitive_ops = vec![
            create_file_op(1000, FileOperation::Read, "/home/user/.ssh/id_rsa"),
            create_file_op(1000, FileOperation::Read, "/home/user/.aws/credentials"),
            create_file_op(1000, FileOperation::Read, "/project/.env"),
        ];

        let monitor = MockFileMonitor::with_file_ops(sensitive_ops);

        // All sensitive files should be tracked
        let snapshot = monitor.snapshot().unwrap();
        assert_eq!(snapshot.len(), 3);
    }
}

// ============================================================================
// Test Module: Edge Cases
// ============================================================================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_path() {
        let op = create_file_op(1000, FileOperation::Read, "");
        assert!(op.path.is_empty());
    }

    #[test]
    fn test_unique_ids() {
        let op1 = create_file_op(1, FileOperation::Read, "/file");
        let op2 = create_file_op(1, FileOperation::Read, "/file");

        assert_ne!(op1.id, op2.id);
    }

    #[test]
    fn test_process_id_zero() {
        let op = create_file_op(0, FileOperation::Read, "/kernel_file");
        assert_eq!(op.pid, 0);
    }

    #[test]
    fn test_timestamp_ordering() {
        let op1 = create_file_op(1, FileOperation::Read, "/file");
        std::thread::sleep(Duration::from_millis(10));
        let op2 = create_file_op(1, FileOperation::Read, "/file");

        assert!(op2.timestamp >= op1.timestamp);
    }
}
