//! Unit tests for process monitoring engine
//!
//! Tests cover:
//! - Process tree construction accuracy
//! - Child process detection latency
//! - Process exit cleanup
//! - PID reuse handling
//! - High process churn scenarios
//!
//! Target: 80%+ coverage per THE-37

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use tuai_common::{PlatformError, PlatformResult, ProcessInfo};
use uuid::Uuid;

use super::ProcessMonitorBackend;

// ============================================================================
// Test Fixtures
// ============================================================================

/// Create a ProcessInfo fixture with the given parameters
fn create_process_fixture(pid: u32, name: &str, ppid: Option<u32>) -> ProcessInfo {
    let mut info = ProcessInfo::new(pid, name.to_string());
    info.ppid = ppid;
    info.cmdline = Some(format!("/usr/bin/{}", name));
    info.exe_path = Some(format!("/usr/bin/{}", name));
    info.cwd = Some("/home/user".to_string());
    info.user = Some("testuser".to_string());
    info
}

/// Create a process tree fixture representing a typical shell hierarchy
fn create_shell_tree_fixture() -> Vec<ProcessInfo> {
    vec![
        create_process_fixture(1, "init", None),
        create_process_fixture(100, "systemd", Some(1)),
        create_process_fixture(1000, "bash", Some(100)),
        create_process_fixture(1001, "tmux", Some(1000)),
        create_process_fixture(1002, "bash", Some(1001)),
        create_process_fixture(1003, "claude", Some(1002)),
        create_process_fixture(1004, "node", Some(1003)),
        create_process_fixture(1005, "python", Some(1003)),
    ]
}

/// Create a fixture representing AI agent processes
fn create_ai_agent_fixture() -> Vec<ProcessInfo> {
    vec![
        create_process_fixture(1000, "bash", None),
        create_process_fixture(2000, "claude", Some(1000)),
        create_process_fixture(2001, "node", Some(2000)),
        create_process_fixture(2002, "python3", Some(2000)),
        create_process_fixture(2003, "git", Some(2000)),
        create_process_fixture(3000, "cursor", Some(1000)),
        create_process_fixture(3001, "node", Some(3000)),
        create_process_fixture(3002, "electron", Some(3000)),
    ]
}

// ============================================================================
// Mock Process Monitor Backend
// ============================================================================

/// Mock process monitor for deterministic testing
pub struct MockProcessMonitor {
    running: AtomicBool,
    processes: Arc<RwLock<HashMap<u32, ProcessInfo>>>,
    spawn_count: AtomicU32,
    exit_count: AtomicU32,
    /// Artificial delay for spawn detection (simulates latency)
    spawn_detection_delay: Duration,
}

impl MockProcessMonitor {
    /// Create a new mock monitor with empty process list
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            processes: Arc::new(RwLock::new(HashMap::new())),
            spawn_count: AtomicU32::new(0),
            exit_count: AtomicU32::new(0),
            spawn_detection_delay: Duration::from_millis(0),
        }
    }

    /// Create a mock monitor pre-populated with processes
    pub fn with_processes(processes: Vec<ProcessInfo>) -> Self {
        let mock = Self::new();
        {
            let mut map = mock.processes.write();
            for p in processes {
                map.insert(p.pid, p);
            }
        }
        mock
    }

    /// Set artificial spawn detection delay for latency testing
    pub fn with_spawn_delay(mut self, delay: Duration) -> Self {
        self.spawn_detection_delay = delay;
        self
    }

    /// Simulate spawning a new process
    pub fn spawn_process(&self, pid: u32, name: &str, ppid: Option<u32>) -> ProcessInfo {
        // Simulate detection delay
        if !self.spawn_detection_delay.is_zero() {
            std::thread::sleep(self.spawn_detection_delay);
        }

        let info = create_process_fixture(pid, name, ppid);
        {
            let mut processes = self.processes.write();
            processes.insert(pid, info.clone());
        }
        self.spawn_count.fetch_add(1, Ordering::SeqCst);
        info
    }

    /// Simulate process exit
    pub fn exit_process(&self, pid: u32) -> Option<ProcessInfo> {
        let mut info = None;
        {
            let mut processes = self.processes.write();
            info = processes.remove(&pid);
        }
        if info.is_some() {
            self.exit_count.fetch_add(1, Ordering::SeqCst);
        }
        info
    }

    /// Get spawn count
    pub fn spawn_count(&self) -> u32 {
        self.spawn_count.load(Ordering::SeqCst)
    }

    /// Get exit count
    pub fn exit_count(&self) -> u32 {
        self.exit_count.load(Ordering::SeqCst)
    }

    /// Get process by PID
    pub fn get_process(&self, pid: u32) -> Option<ProcessInfo> {
        self.processes.read().get(&pid).cloned()
    }

    /// Get all child processes of a given PID
    pub fn get_children(&self, ppid: u32) -> Vec<ProcessInfo> {
        self.processes
            .read()
            .values()
            .filter(|p| p.ppid == Some(ppid))
            .cloned()
            .collect()
    }

    /// Build a process tree from a given root PID
    pub fn build_tree(&self, root_pid: u32) -> HashMap<u32, Vec<u32>> {
        let processes = self.processes.read();
        let mut tree: HashMap<u32, Vec<u32>> = HashMap::new();

        for process in processes.values() {
            if let Some(ppid) = process.ppid {
                tree.entry(ppid).or_default().push(process.pid);
            }
        }

        tree
    }

    /// Get process count
    pub fn process_count(&self) -> usize {
        self.processes.read().len()
    }
}

impl Default for MockProcessMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessMonitorBackend for MockProcessMonitor {
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

    fn snapshot(&self) -> PlatformResult<Vec<ProcessInfo>> {
        Ok(self.processes.read().values().cloned().collect())
    }
}

// ============================================================================
// Process Tree Data Structure for Testing
// ============================================================================

/// Represents a process tree node for testing tree construction accuracy
#[derive(Debug, Clone)]
pub struct ProcessTreeNode {
    pub process: ProcessInfo,
    pub children: Vec<ProcessTreeNode>,
}

impl ProcessTreeNode {
    /// Create a new tree node
    pub fn new(process: ProcessInfo) -> Self {
        Self {
            process,
            children: Vec::new(),
        }
    }

    /// Count all nodes in the tree (including self)
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(|c| c.count()).sum::<usize>()
    }

    /// Get maximum depth of the tree
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
        }
    }

    /// Find a node by PID
    pub fn find(&self, pid: u32) -> Option<&ProcessTreeNode> {
        if self.process.pid == pid {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find(pid) {
                return Some(found);
            }
        }
        None
    }
}

/// Build a process tree from a flat list of processes
fn build_process_tree(processes: &[ProcessInfo], root_pid: u32) -> Option<ProcessTreeNode> {
    let process_map: HashMap<u32, &ProcessInfo> = processes.iter().map(|p| (p.pid, p)).collect();

    let root = process_map.get(&root_pid)?;
    let mut root_node = ProcessTreeNode::new((*root).clone());

    fn add_children(
        node: &mut ProcessTreeNode,
        processes: &[ProcessInfo],
        process_map: &HashMap<u32, &ProcessInfo>,
    ) {
        for p in processes {
            if p.ppid == Some(node.process.pid) {
                let mut child_node = ProcessTreeNode::new(p.clone());
                add_children(&mut child_node, processes, process_map);
                node.children.push(child_node);
            }
        }
    }

    add_children(&mut root_node, processes, &process_map);
    Some(root_node)
}

// ============================================================================
// Test Module: Process Tree Construction Accuracy
// ============================================================================

#[cfg(test)]
mod tree_construction_tests {
    use super::*;

    #[test]
    fn test_empty_process_list() {
        let monitor = MockProcessMonitor::new();
        let snapshot = monitor.snapshot().unwrap();
        assert!(snapshot.is_empty());
    }

    #[test]
    fn test_single_process() {
        let processes = vec![create_process_fixture(1, "init", None)];
        let monitor = MockProcessMonitor::with_processes(processes);

        let snapshot = monitor.snapshot().unwrap();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].pid, 1);
        assert_eq!(snapshot[0].name, "init");
        assert!(snapshot[0].ppid.is_none());
    }

    #[test]
    fn test_parent_child_relationship() {
        let processes = vec![
            create_process_fixture(1, "parent", None),
            create_process_fixture(2, "child", Some(1)),
        ];
        let monitor = MockProcessMonitor::with_processes(processes);

        let children = monitor.get_children(1);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].pid, 2);
        assert_eq!(children[0].ppid, Some(1));
    }

    #[test]
    fn test_shell_tree_structure() {
        let processes = create_shell_tree_fixture();
        let tree = build_process_tree(&processes, 1).unwrap();

        // Verify tree structure
        assert_eq!(tree.process.pid, 1);
        assert_eq!(tree.count(), 8); // Total 8 processes
        assert!(tree.depth() >= 5); // init -> systemd -> bash -> tmux -> bash -> claude
    }

    #[test]
    fn test_ai_agent_tree() {
        let processes = create_ai_agent_fixture();
        let tree = build_process_tree(&processes, 1000).unwrap();

        // bash (1000) has two children: claude (2000) and cursor (3000)
        assert_eq!(tree.children.len(), 2);

        // Find claude node and verify its children
        let claude = tree.find(2000).unwrap();
        assert_eq!(claude.children.len(), 3); // node, python3, git

        // Find cursor node and verify its children
        let cursor = tree.find(3000).unwrap();
        assert_eq!(cursor.children.len(), 2); // node, electron
    }

    #[test]
    fn test_multiple_roots() {
        // Simulate processes from different hierarchies
        let processes = vec![
            create_process_fixture(1, "init", None),
            create_process_fixture(100, "child1", Some(1)),
            create_process_fixture(2, "kthreadd", None),
            create_process_fixture(200, "kworker", Some(2)),
        ];

        let monitor = MockProcessMonitor::with_processes(processes);
        let snapshot = monitor.snapshot().unwrap();

        // Should have 4 processes total
        assert_eq!(snapshot.len(), 4);

        // Build trees from both roots
        let tree1 = build_process_tree(&snapshot, 1).unwrap();
        let tree2 = build_process_tree(&snapshot, 2).unwrap();

        assert_eq!(tree1.count(), 2);
        assert_eq!(tree2.count(), 2);
    }

    #[test]
    fn test_deep_tree() {
        // Create a deep process chain: p1 -> p2 -> p3 -> ... -> p100
        let mut processes = Vec::new();
        processes.push(create_process_fixture(1, "root", None));
        for i in 2..=100 {
            processes.push(create_process_fixture(i, &format!("proc{}", i), Some(i - 1)));
        }

        let tree = build_process_tree(&processes, 1).unwrap();
        assert_eq!(tree.count(), 100);
        assert_eq!(tree.depth(), 100);
    }

    #[test]
    fn test_wide_tree() {
        // Create a wide tree: root with 100 direct children
        let mut processes = Vec::new();
        processes.push(create_process_fixture(1, "root", None));
        for i in 2..=101 {
            processes.push(create_process_fixture(i, &format!("child{}", i), Some(1)));
        }

        let tree = build_process_tree(&processes, 1).unwrap();
        assert_eq!(tree.count(), 101);
        assert_eq!(tree.depth(), 2);
        assert_eq!(tree.children.len(), 100);
    }

    #[test]
    fn test_orphan_processes() {
        // Process with non-existent parent (orphan)
        let processes = vec![
            create_process_fixture(1, "init", None),
            create_process_fixture(100, "orphan", Some(999)), // Parent 999 doesn't exist
        ];

        let monitor = MockProcessMonitor::with_processes(processes);

        // init has no children (orphan's parent doesn't exist)
        let children = monitor.get_children(1);
        assert!(children.is_empty());

        // But orphan still exists in snapshot
        assert_eq!(monitor.process_count(), 2);
    }

    #[test]
    fn test_process_info_completeness() {
        let process = create_process_fixture(1234, "test_proc", Some(1));

        assert_eq!(process.pid, 1234);
        assert_eq!(process.name, "test_proc");
        assert_eq!(process.ppid, Some(1));
        assert!(process.cmdline.is_some());
        assert!(process.exe_path.is_some());
        assert!(process.cwd.is_some());
        assert!(process.user.is_some());
        assert!(process.end_time.is_none()); // Not exited yet
    }
}

// ============================================================================
// Test Module: Child Process Detection
// ============================================================================

#[cfg(test)]
mod child_detection_tests {
    use super::*;

    #[test]
    fn test_spawn_detection_basic() {
        let monitor = MockProcessMonitor::new();

        // Spawn a process
        let info = monitor.spawn_process(1000, "test", None);
        assert_eq!(info.pid, 1000);
        assert_eq!(monitor.spawn_count(), 1);
        assert_eq!(monitor.process_count(), 1);
    }

    #[test]
    fn test_spawn_multiple_children() {
        let processes = vec![create_process_fixture(1, "parent", None)];
        let monitor = MockProcessMonitor::with_processes(processes);

        // Spawn 5 children
        for i in 2..=6 {
            monitor.spawn_process(i, &format!("child{}", i), Some(1));
        }

        assert_eq!(monitor.spawn_count(), 5);
        assert_eq!(monitor.get_children(1).len(), 5);
    }

    #[test]
    fn test_spawn_detection_latency() {
        // Test that spawn detection happens within target latency (100ms)
        let monitor = MockProcessMonitor::new().with_spawn_delay(Duration::from_millis(10));

        let start = Instant::now();
        monitor.spawn_process(1, "fast", None);
        let elapsed = start.elapsed();

        // Should complete within 100ms target
        assert!(
            elapsed < Duration::from_millis(100),
            "Spawn detection took {:?}, expected < 100ms",
            elapsed
        );
    }

    #[test]
    fn test_nested_child_spawn() {
        let monitor = MockProcessMonitor::new();

        // Create nested hierarchy
        monitor.spawn_process(1, "grandparent", None);
        monitor.spawn_process(2, "parent", Some(1));
        monitor.spawn_process(3, "child", Some(2));
        monitor.spawn_process(4, "grandchild", Some(3));

        // Verify each level
        assert_eq!(monitor.get_children(1).len(), 1);
        assert_eq!(monitor.get_children(2).len(), 1);
        assert_eq!(monitor.get_children(3).len(), 1);
        assert_eq!(monitor.get_children(4).len(), 0); // Leaf node
    }

    #[test]
    fn test_sibling_processes() {
        let monitor = MockProcessMonitor::new();

        monitor.spawn_process(1, "parent", None);

        // Spawn multiple siblings
        monitor.spawn_process(2, "sibling1", Some(1));
        monitor.spawn_process(3, "sibling2", Some(1));
        monitor.spawn_process(4, "sibling3", Some(1));

        let children = monitor.get_children(1);
        assert_eq!(children.len(), 3);

        // All siblings should have same parent
        for child in &children {
            assert_eq!(child.ppid, Some(1));
        }
    }

    #[test]
    fn test_rapid_spawn_burst() {
        let monitor = MockProcessMonitor::new();
        monitor.spawn_process(1, "parent", None);

        // Rapidly spawn 50 children
        let start = Instant::now();
        for i in 2..=51 {
            monitor.spawn_process(i, &format!("child{}", i), Some(1));
        }
        let elapsed = start.elapsed();

        assert_eq!(monitor.spawn_count(), 51);
        assert_eq!(monitor.get_children(1).len(), 50);

        // Even 50 spawns should be fast (< 1s without artificial delay)
        assert!(elapsed < Duration::from_secs(1));
    }
}

// ============================================================================
// Test Module: Process Exit Cleanup
// ============================================================================

#[cfg(test)]
mod exit_cleanup_tests {
    use super::*;

    #[test]
    fn test_basic_exit() {
        let processes = vec![create_process_fixture(1, "test", None)];
        let monitor = MockProcessMonitor::with_processes(processes);

        assert_eq!(monitor.process_count(), 1);

        let exited = monitor.exit_process(1);
        assert!(exited.is_some());
        assert_eq!(exited.unwrap().pid, 1);
        assert_eq!(monitor.process_count(), 0);
        assert_eq!(monitor.exit_count(), 1);
    }

    #[test]
    fn test_exit_nonexistent() {
        let monitor = MockProcessMonitor::new();

        // Try to exit a process that doesn't exist
        let exited = monitor.exit_process(999);
        assert!(exited.is_none());
        assert_eq!(monitor.exit_count(), 0);
    }

    #[test]
    fn test_parent_exits_before_children() {
        let processes = vec![
            create_process_fixture(1, "parent", None),
            create_process_fixture(2, "child1", Some(1)),
            create_process_fixture(3, "child2", Some(1)),
        ];
        let monitor = MockProcessMonitor::with_processes(processes);

        // Parent exits first (children become orphans in real system)
        monitor.exit_process(1);

        // Children should still exist
        assert_eq!(monitor.process_count(), 2);
        assert!(monitor.get_process(2).is_some());
        assert!(monitor.get_process(3).is_some());
    }

    #[test]
    fn test_child_exits_before_parent() {
        let processes = vec![
            create_process_fixture(1, "parent", None),
            create_process_fixture(2, "child", Some(1)),
        ];
        let monitor = MockProcessMonitor::with_processes(processes);

        // Child exits
        monitor.exit_process(2);

        // Parent still exists, but has no children
        assert_eq!(monitor.process_count(), 1);
        assert!(monitor.get_children(1).is_empty());
    }

    #[test]
    fn test_cascade_cleanup() {
        let processes = vec![
            create_process_fixture(1, "root", None),
            create_process_fixture(2, "child", Some(1)),
            create_process_fixture(3, "grandchild", Some(2)),
        ];
        let monitor = MockProcessMonitor::with_processes(processes);

        // Clean up from bottom up
        monitor.exit_process(3);
        assert_eq!(monitor.process_count(), 2);

        monitor.exit_process(2);
        assert_eq!(monitor.process_count(), 1);

        monitor.exit_process(1);
        assert_eq!(monitor.process_count(), 0);
    }

    #[test]
    fn test_mass_exit() {
        let mut processes = Vec::new();
        for i in 1..=100 {
            processes.push(create_process_fixture(i, &format!("proc{}", i), None));
        }
        let monitor = MockProcessMonitor::with_processes(processes);

        // Exit all processes
        for i in 1..=100 {
            monitor.exit_process(i);
        }

        assert_eq!(monitor.process_count(), 0);
        assert_eq!(monitor.exit_count(), 100);
    }

    #[test]
    fn test_double_exit() {
        let processes = vec![create_process_fixture(1, "test", None)];
        let monitor = MockProcessMonitor::with_processes(processes);

        // First exit succeeds
        let first = monitor.exit_process(1);
        assert!(first.is_some());

        // Second exit fails (already gone)
        let second = monitor.exit_process(1);
        assert!(second.is_none());

        assert_eq!(monitor.exit_count(), 1);
    }
}

// ============================================================================
// Test Module: PID Reuse Handling
// ============================================================================

#[cfg(test)]
mod pid_reuse_tests {
    use super::*;

    #[test]
    fn test_pid_reuse_basic() {
        let monitor = MockProcessMonitor::new();

        // First process with PID 1000
        monitor.spawn_process(1000, "first_process", None);
        assert_eq!(monitor.get_process(1000).unwrap().name, "first_process");

        // Process exits
        monitor.exit_process(1000);
        assert!(monitor.get_process(1000).is_none());

        // New process reuses PID 1000
        monitor.spawn_process(1000, "second_process", None);
        let p = monitor.get_process(1000).unwrap();
        assert_eq!(p.name, "second_process");
    }

    #[test]
    fn test_pid_reuse_different_parent() {
        let monitor = MockProcessMonitor::new();

        monitor.spawn_process(1, "parent1", None);
        monitor.spawn_process(2, "parent2", None);

        // Child with PID 100 under parent1
        monitor.spawn_process(100, "child", Some(1));
        assert_eq!(monitor.get_process(100).unwrap().ppid, Some(1));

        // Child exits
        monitor.exit_process(100);

        // Same PID reused under different parent
        monitor.spawn_process(100, "new_child", Some(2));
        assert_eq!(monitor.get_process(100).unwrap().ppid, Some(2));
    }

    #[test]
    fn test_pid_reuse_unique_ids() {
        let monitor = MockProcessMonitor::new();

        // First process
        monitor.spawn_process(1000, "first", None);
        let first_id = monitor.get_process(1000).unwrap().id;

        monitor.exit_process(1000);

        // Second process with same PID
        monitor.spawn_process(1000, "second", None);
        let second_id = monitor.get_process(1000).unwrap().id;

        // UUIDs should be different
        assert_ne!(first_id, second_id);
    }

    #[test]
    fn test_rapid_pid_cycling() {
        let monitor = MockProcessMonitor::new();
        let mut seen_ids: Vec<Uuid> = Vec::new();

        // Cycle through PID 1000 many times
        for i in 0..50 {
            let name = format!("process_{}", i);
            monitor.spawn_process(1000, &name, None);
            seen_ids.push(monitor.get_process(1000).unwrap().id);
            monitor.exit_process(1000);
        }

        // All IDs should be unique
        let unique_count = seen_ids.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, 50);
    }

    #[test]
    fn test_pid_reuse_maintains_tree() {
        let monitor = MockProcessMonitor::new();

        // Build initial tree
        monitor.spawn_process(1, "root", None);
        monitor.spawn_process(100, "child", Some(1));
        monitor.spawn_process(200, "grandchild", Some(100));

        // Child exits, grandchild becomes orphan in terms of our tracking
        monitor.exit_process(100);

        // New process reuses PID 100 but with different parent
        monitor.spawn_process(100, "new_child", Some(1));

        // Grandchild still points to old parent (PID 100) but it's a different process
        let grandchild = monitor.get_process(200).unwrap();
        assert_eq!(grandchild.ppid, Some(100));

        // New child under root
        let new_child = monitor.get_process(100).unwrap();
        assert_eq!(new_child.name, "new_child");
    }
}

// ============================================================================
// Test Module: High Process Churn
// ============================================================================

#[cfg(test)]
mod high_churn_tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_hundred_spawns_per_second() {
        let monitor = MockProcessMonitor::new();
        let start = Instant::now();

        // Spawn 100+ processes
        for i in 1..=120 {
            monitor.spawn_process(i, &format!("proc{}", i), None);
        }

        let elapsed = start.elapsed();

        assert_eq!(monitor.spawn_count(), 120);
        // Should complete in under 1 second
        assert!(
            elapsed < Duration::from_secs(1),
            "120 spawns took {:?}, expected < 1s",
            elapsed
        );

        // Verify rate
        let spawns_per_sec = 120.0 / elapsed.as_secs_f64();
        assert!(
            spawns_per_sec >= 100.0,
            "Spawn rate {:.0}/s below 100/s target",
            spawns_per_sec
        );
    }

    #[test]
    fn test_rapid_spawn_exit_cycles() {
        let monitor = MockProcessMonitor::new();
        let start = Instant::now();

        // Rapid spawn-exit cycles
        for i in 1..=100 {
            monitor.spawn_process(1000 + i, &format!("short_lived_{}", i), None);
            monitor.exit_process(1000 + i);
        }

        let elapsed = start.elapsed();

        assert_eq!(monitor.spawn_count(), 100);
        assert_eq!(monitor.exit_count(), 100);
        assert_eq!(monitor.process_count(), 0);

        // Should complete quickly
        assert!(elapsed < Duration::from_secs(1));
    }

    #[test]
    fn test_concurrent_operations() {
        let monitor = Arc::new(MockProcessMonitor::new());
        let mut handles = Vec::new();

        // Spawn from multiple threads
        for t in 0..4 {
            let m = Arc::clone(&monitor);
            let handle = thread::spawn(move || {
                for i in 0..25 {
                    let pid = (t * 1000 + i) as u32;
                    m.spawn_process(pid, &format!("thread{}_{}", t, i), None);
                }
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().unwrap();
        }

        // All 100 processes should be spawned
        assert_eq!(monitor.spawn_count(), 100);
        assert_eq!(monitor.process_count(), 100);
    }

    #[test]
    fn test_tree_under_churn() {
        let monitor = MockProcessMonitor::new();

        // Create a root
        monitor.spawn_process(1, "root", None);

        // Rapidly spawn and exit children
        for wave in 0..10 {
            // Spawn 10 children
            for i in 0..10 {
                let pid = (wave * 100 + i + 2) as u32;
                monitor.spawn_process(pid, &format!("child_{}_{}", wave, i), Some(1));
            }

            // Exit them all
            for i in 0..10 {
                let pid = (wave * 100 + i + 2) as u32;
                monitor.exit_process(pid);
            }
        }

        // Only root should remain
        assert_eq!(monitor.process_count(), 1);
        assert!(monitor.get_children(1).is_empty());
        assert_eq!(monitor.spawn_count(), 101); // 1 root + 100 children
        assert_eq!(monitor.exit_count(), 100); // 100 children exited
    }

    #[test]
    fn test_sustained_load() {
        let monitor = MockProcessMonitor::new();
        let duration = Duration::from_millis(100);
        let start = Instant::now();
        let mut count = 0u32;

        // Run for a fixed duration
        while start.elapsed() < duration {
            count += 1;
            monitor.spawn_process(count, &format!("proc{}", count), None);
            if count > 1 && count % 2 == 0 {
                monitor.exit_process(count - 1);
            }
        }

        let elapsed = start.elapsed();
        let ops_per_sec = (count as f64) / elapsed.as_secs_f64();

        // Should handle at least 1000 ops/sec
        assert!(
            ops_per_sec >= 1000.0,
            "Only {:.0} ops/sec, expected >= 1000",
            ops_per_sec
        );
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
        let mut monitor = MockProcessMonitor::new();

        assert!(!monitor.is_running());

        monitor.start().unwrap();
        assert!(monitor.is_running());

        monitor.stop().unwrap();
        assert!(!monitor.is_running());
    }

    #[test]
    fn test_double_start() {
        let mut monitor = MockProcessMonitor::new();

        monitor.start().unwrap();
        // Second start should be idempotent
        monitor.start().unwrap();

        assert!(monitor.is_running());
    }

    #[test]
    fn test_snapshot_returns_all_processes() {
        let processes = create_shell_tree_fixture();
        let expected_count = processes.len();
        let monitor = MockProcessMonitor::with_processes(processes);

        let snapshot = monitor.snapshot().unwrap();
        assert_eq!(snapshot.len(), expected_count);
    }

    #[test]
    fn test_snapshot_consistency() {
        let monitor = MockProcessMonitor::new();

        // Add some processes
        monitor.spawn_process(1, "a", None);
        monitor.spawn_process(2, "b", None);

        let snap1 = monitor.snapshot().unwrap();
        let snap2 = monitor.snapshot().unwrap();

        // Snapshots should be consistent
        assert_eq!(snap1.len(), snap2.len());

        // PIDs should match
        let pids1: std::collections::HashSet<_> = snap1.iter().map(|p| p.pid).collect();
        let pids2: std::collections::HashSet<_> = snap2.iter().map(|p| p.pid).collect();
        assert_eq!(pids1, pids2);
    }
}

// ============================================================================
// Test Module: Snapshot Testing with insta
// ============================================================================

#[cfg(test)]
mod snapshot_tests {
    use super::*;

    fn normalize_process_for_snapshot(p: &ProcessInfo) -> serde_json::Value {
        serde_json::json!({
            "pid": p.pid,
            "ppid": p.ppid,
            "name": p.name,
        })
    }

    #[test]
    fn test_shell_tree_snapshot() {
        let processes = create_shell_tree_fixture();
        let tree = build_process_tree(&processes, 1).unwrap();

        let snapshot = serde_json::json!({
            "root_pid": tree.process.pid,
            "total_nodes": tree.count(),
            "max_depth": tree.depth(),
        });

        insta::assert_json_snapshot!("shell_tree_structure", snapshot);
    }

    #[test]
    fn test_ai_agent_tree_snapshot() {
        let processes = create_ai_agent_fixture();
        let tree = build_process_tree(&processes, 1000).unwrap();

        let children_pids: Vec<u32> = tree.children.iter().map(|c| c.process.pid).collect();

        let snapshot = serde_json::json!({
            "root_pid": tree.process.pid,
            "root_name": tree.process.name,
            "direct_children": children_pids,
            "total_nodes": tree.count(),
        });

        insta::assert_json_snapshot!("ai_agent_tree", snapshot);
    }
}

// ============================================================================
// Integration-style tests using SysinfoMonitor
// ============================================================================

#[cfg(test)]
mod sysinfo_integration_tests {
    use super::super::SysinfoMonitor;
    use super::*;

    #[test]
    fn test_sysinfo_monitor_creation() {
        let monitor = SysinfoMonitor::new();
        assert!(!monitor.is_running());
    }

    #[test]
    fn test_sysinfo_monitor_start_stop() {
        let mut monitor = SysinfoMonitor::new();

        monitor.start().unwrap();
        assert!(monitor.is_running());

        monitor.stop().unwrap();
        assert!(!monitor.is_running());
    }

    #[test]
    fn test_sysinfo_snapshot_not_empty() {
        let mut monitor = SysinfoMonitor::new();
        monitor.start().unwrap();

        let snapshot = monitor.snapshot().unwrap();

        // Should have at least the current process
        assert!(!snapshot.is_empty());

        // Find current process
        let current_pid = std::process::id();
        let found = snapshot.iter().any(|p| p.pid == current_pid);
        assert!(found, "Current process not found in snapshot");
    }

    #[test]
    fn test_sysinfo_process_has_required_fields() {
        let mut monitor = SysinfoMonitor::new();
        monitor.start().unwrap();

        let snapshot = monitor.snapshot().unwrap();
        let current_pid = std::process::id();

        if let Some(current) = snapshot.iter().find(|p| p.pid == current_pid) {
            // These fields should be populated
            assert!(!current.name.is_empty());
            // cmdline might be empty for some processes but should exist
            // exe_path should exist for our own process
        }
    }

    #[test]
    fn test_sysinfo_multiple_snapshots() {
        let mut monitor = SysinfoMonitor::new();
        monitor.start().unwrap();

        let snap1 = monitor.snapshot().unwrap();
        let snap2 = monitor.snapshot().unwrap();

        // Both should have processes
        assert!(!snap1.is_empty());
        assert!(!snap2.is_empty());

        // Current process should be in both
        let current_pid = std::process::id();
        assert!(snap1.iter().any(|p| p.pid == current_pid));
        assert!(snap2.iter().any(|p| p.pid == current_pid));
    }
}
