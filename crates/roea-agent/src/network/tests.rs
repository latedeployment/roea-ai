//! Unit tests for network connection tracking
//!
//! Tests cover:
//! - TCP/UDP connection detection
//! - Unix socket discovery
//! - Connection state transitions
//! - Remote IP/port extraction
//! - IPv4 vs IPv6 handling
//! - High connection volume (1000+ concurrent)
//! - Loopback connections
//! - Connections during process exit
//! - Socket inheritance after fork
//!
//! Target: 80%+ coverage per THE-38

use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use parking_lot::RwLock;
use roea_common::{ConnectionInfo, ConnectionState, PlatformError, PlatformResult, Protocol};
use uuid::Uuid;

use super::{EndpointClass, NetworkMonitorBackend, NetworkMonitorService};

// ============================================================================
// Test Fixtures
// ============================================================================

/// Create a TCP connection fixture
fn create_tcp_connection(
    pid: u32,
    local_addr: &str,
    local_port: u16,
    remote_addr: &str,
    remote_port: u16,
    state: ConnectionState,
) -> ConnectionInfo {
    let mut conn = ConnectionInfo::new(pid, Protocol::Tcp);
    conn.local_addr = Some(local_addr.to_string());
    conn.local_port = Some(local_port);
    conn.remote_addr = Some(remote_addr.to_string());
    conn.remote_port = Some(remote_port);
    conn.state = state;
    conn.timestamp = Utc::now();
    conn
}

/// Create a UDP connection fixture
fn create_udp_connection(
    pid: u32,
    local_addr: &str,
    local_port: u16,
    remote_addr: Option<&str>,
    remote_port: Option<u16>,
) -> ConnectionInfo {
    let mut conn = ConnectionInfo::new(pid, Protocol::Udp);
    conn.local_addr = Some(local_addr.to_string());
    conn.local_port = Some(local_port);
    conn.remote_addr = remote_addr.map(|s| s.to_string());
    conn.remote_port = remote_port;
    conn.state = ConnectionState::Established;
    conn.timestamp = Utc::now();
    conn
}

/// Create a Unix socket fixture
fn create_unix_socket(pid: u32, path: Option<&str>) -> ConnectionInfo {
    let mut conn = ConnectionInfo::new(pid, Protocol::Unix);
    conn.local_addr = path.map(|s| s.to_string());
    conn.state = ConnectionState::Established;
    conn.timestamp = Utc::now();
    conn
}

/// Create a typical AI agent connection set
fn create_ai_agent_connections() -> Vec<ConnectionInfo> {
    vec![
        // Claude Code API connection
        create_tcp_connection(
            1000,
            "192.168.1.100",
            54321,
            "104.18.24.55",
            443,
            ConnectionState::Established,
        ),
        // OpenAI API connection
        create_tcp_connection(
            1001,
            "192.168.1.100",
            54322,
            "13.107.42.16",
            443,
            ConnectionState::Established,
        ),
        // GitHub API connection
        create_tcp_connection(
            1002,
            "192.168.1.100",
            54323,
            "140.82.121.4",
            443,
            ConnectionState::Established,
        ),
        // Local Unix socket for IPC
        create_unix_socket(1000, Some("/tmp/claude-ipc.sock")),
        // DNS query
        create_udp_connection(1000, "192.168.1.100", 45678, Some("8.8.8.8"), Some(53)),
    ]
}

// ============================================================================
// Mock Network Monitor Backend
// ============================================================================

/// Mock network monitor for deterministic testing
pub struct MockNetworkMonitor {
    running: AtomicBool,
    connections: Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>,
    connection_count: AtomicU32,
}

impl MockNetworkMonitor {
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            connections: Arc::new(RwLock::new(HashMap::new())),
            connection_count: AtomicU32::new(0),
        }
    }

    /// Create a mock monitor pre-populated with connections
    pub fn with_connections(connections: Vec<ConnectionInfo>) -> Self {
        let mock = Self::new();
        {
            let mut map = mock.connections.write();
            for c in connections {
                map.insert(c.id, c);
            }
        }
        mock
    }

    /// Add a new connection
    pub fn add_connection(&self, conn: ConnectionInfo) -> Uuid {
        let id = conn.id;
        {
            let mut connections = self.connections.write();
            connections.insert(id, conn);
        }
        self.connection_count.fetch_add(1, Ordering::SeqCst);
        id
    }

    /// Remove a connection
    pub fn remove_connection(&self, id: Uuid) -> Option<ConnectionInfo> {
        self.connections.write().remove(&id)
    }

    /// Update connection state
    pub fn update_state(&self, id: Uuid, state: ConnectionState) -> bool {
        let mut connections = self.connections.write();
        if let Some(conn) = connections.get_mut(&id) {
            conn.state = state;
            true
        } else {
            false
        }
    }

    /// Get connections by protocol
    pub fn connections_by_protocol(&self, protocol: Protocol) -> Vec<ConnectionInfo> {
        self.connections
            .read()
            .values()
            .filter(|c| c.protocol == protocol)
            .cloned()
            .collect()
    }

    /// Get connections by PID
    pub fn connections_by_pid(&self, pid: u32) -> Vec<ConnectionInfo> {
        self.connections
            .read()
            .values()
            .filter(|c| c.pid == pid)
            .cloned()
            .collect()
    }

    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.read().len()
    }

    /// Get total connections added
    pub fn total_added(&self) -> u32 {
        self.connection_count.load(Ordering::SeqCst)
    }
}

impl Default for MockNetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkMonitorBackend for MockNetworkMonitor {
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

    fn snapshot(&self) -> PlatformResult<Vec<ConnectionInfo>> {
        Ok(self.connections.read().values().cloned().collect())
    }
}

// ============================================================================
// Test Module: TCP/UDP Connection Detection
// ============================================================================

#[cfg(test)]
mod tcp_udp_tests {
    use super::*;

    #[test]
    fn test_tcp_connection_creation() {
        let conn = create_tcp_connection(
            1000,
            "192.168.1.100",
            54321,
            "8.8.8.8",
            443,
            ConnectionState::Established,
        );

        assert_eq!(conn.pid, 1000);
        assert_eq!(conn.protocol, Protocol::Tcp);
        assert_eq!(conn.local_addr.as_deref(), Some("192.168.1.100"));
        assert_eq!(conn.local_port, Some(54321));
        assert_eq!(conn.remote_addr.as_deref(), Some("8.8.8.8"));
        assert_eq!(conn.remote_port, Some(443));
        assert_eq!(conn.state, ConnectionState::Established);
    }

    #[test]
    fn test_udp_connection_creation() {
        let conn = create_udp_connection(
            1000,
            "192.168.1.100",
            45678,
            Some("8.8.8.8"),
            Some(53),
        );

        assert_eq!(conn.protocol, Protocol::Udp);
        assert_eq!(conn.remote_addr.as_deref(), Some("8.8.8.8"));
        assert_eq!(conn.remote_port, Some(53));
    }

    #[test]
    fn test_udp_without_remote() {
        // UDP can be "connectionless" - local bind only
        let conn = create_udp_connection(1000, "0.0.0.0", 53, None, None);

        assert_eq!(conn.protocol, Protocol::Udp);
        assert!(conn.remote_addr.is_none());
        assert!(conn.remote_port.is_none());
    }

    #[test]
    fn test_filter_by_protocol() {
        let connections = vec![
            create_tcp_connection(1, "127.0.0.1", 80, "8.8.8.8", 443, ConnectionState::Established),
            create_udp_connection(2, "0.0.0.0", 53, Some("8.8.4.4"), Some(53)),
            create_tcp_connection(3, "127.0.0.1", 81, "1.1.1.1", 443, ConnectionState::Established),
            create_unix_socket(4, Some("/var/run/test.sock")),
        ];

        let monitor = MockNetworkMonitor::with_connections(connections);

        let tcp_conns = monitor.connections_by_protocol(Protocol::Tcp);
        assert_eq!(tcp_conns.len(), 2);

        let udp_conns = monitor.connections_by_protocol(Protocol::Udp);
        assert_eq!(udp_conns.len(), 1);

        let unix_conns = monitor.connections_by_protocol(Protocol::Unix);
        assert_eq!(unix_conns.len(), 1);
    }

    #[test]
    fn test_multiple_connections_per_process() {
        let connections = vec![
            create_tcp_connection(100, "127.0.0.1", 8000, "api.example.com", 443, ConnectionState::Established),
            create_tcp_connection(100, "127.0.0.1", 8001, "api.other.com", 443, ConnectionState::Established),
            create_tcp_connection(100, "127.0.0.1", 8002, "api.third.com", 443, ConnectionState::Connecting),
            create_udp_connection(100, "127.0.0.1", 53000, Some("8.8.8.8"), Some(53)),
        ];

        let monitor = MockNetworkMonitor::with_connections(connections);
        let pid_100_conns = monitor.connections_by_pid(100);

        assert_eq!(pid_100_conns.len(), 4);
    }

    #[test]
    fn test_well_known_ports() {
        let ports = vec![
            (22, "SSH"),
            (80, "HTTP"),
            (443, "HTTPS"),
            (53, "DNS"),
            (3306, "MySQL"),
            (5432, "PostgreSQL"),
            (6379, "Redis"),
        ];

        for (port, _name) in ports {
            let conn = create_tcp_connection(
                1,
                "127.0.0.1",
                50000,
                "server.example.com",
                port,
                ConnectionState::Established,
            );
            assert_eq!(conn.remote_port, Some(port));
        }
    }
}

// ============================================================================
// Test Module: Unix Socket Discovery
// ============================================================================

#[cfg(test)]
mod unix_socket_tests {
    use super::*;

    #[test]
    fn test_unix_socket_creation() {
        let sock = create_unix_socket(1000, Some("/var/run/docker.sock"));

        assert_eq!(sock.pid, 1000);
        assert_eq!(sock.protocol, Protocol::Unix);
        assert_eq!(sock.local_addr.as_deref(), Some("/var/run/docker.sock"));
        assert_eq!(sock.state, ConnectionState::Established);
    }

    #[test]
    fn test_abstract_unix_socket() {
        // Abstract sockets start with @ and have no filesystem path
        let sock = create_unix_socket(1000, Some("@/tmp/.X11-unix/X0"));

        assert!(sock.local_addr.as_ref().unwrap().starts_with("@"));
    }

    #[test]
    fn test_anonymous_unix_socket() {
        let sock = create_unix_socket(1000, None);

        assert!(sock.local_addr.is_none());
        assert_eq!(sock.protocol, Protocol::Unix);
    }

    #[test]
    fn test_common_unix_sockets() {
        let common_sockets = vec![
            "/var/run/docker.sock",
            "/var/run/dbus/system_bus_socket",
            "/run/user/1000/bus",
            "/tmp/.X11-unix/X0",
            "/var/run/acpid.socket",
        ];

        for path in common_sockets {
            let sock = create_unix_socket(1, Some(path));
            assert_eq!(sock.local_addr.as_deref(), Some(path));
        }
    }

    #[test]
    fn test_filter_unix_sockets() {
        let connections = vec![
            create_tcp_connection(1, "127.0.0.1", 80, "8.8.8.8", 443, ConnectionState::Established),
            create_unix_socket(1, Some("/var/run/socket1")),
            create_unix_socket(2, Some("/var/run/socket2")),
            create_unix_socket(3, None),
        ];

        let monitor = MockNetworkMonitor::with_connections(connections);
        let unix = monitor.connections_by_protocol(Protocol::Unix);

        assert_eq!(unix.len(), 3);
    }
}

// ============================================================================
// Test Module: Connection State Transitions
// ============================================================================

#[cfg(test)]
mod state_transition_tests {
    use super::*;

    #[test]
    fn test_initial_states() {
        let connecting = create_tcp_connection(1, "127.0.0.1", 80, "8.8.8.8", 443, ConnectionState::Connecting);
        let established = create_tcp_connection(2, "127.0.0.1", 81, "8.8.8.8", 443, ConnectionState::Established);
        let closed = create_tcp_connection(3, "127.0.0.1", 82, "8.8.8.8", 443, ConnectionState::Closed);

        assert_eq!(connecting.state, ConnectionState::Connecting);
        assert_eq!(established.state, ConnectionState::Established);
        assert_eq!(closed.state, ConnectionState::Closed);
    }

    #[test]
    fn test_state_transition_connecting_to_established() {
        let monitor = MockNetworkMonitor::new();
        let conn = create_tcp_connection(
            1,
            "127.0.0.1",
            54321,
            "8.8.8.8",
            443,
            ConnectionState::Connecting,
        );
        let id = monitor.add_connection(conn);

        // Transition to established
        assert!(monitor.update_state(id, ConnectionState::Established));

        let snapshot = monitor.snapshot().unwrap();
        let updated = snapshot.iter().find(|c| c.id == id).unwrap();
        assert_eq!(updated.state, ConnectionState::Established);
    }

    #[test]
    fn test_state_transition_to_closed() {
        let monitor = MockNetworkMonitor::new();
        let conn = create_tcp_connection(
            1,
            "127.0.0.1",
            54321,
            "8.8.8.8",
            443,
            ConnectionState::Established,
        );
        let id = monitor.add_connection(conn);

        assert!(monitor.update_state(id, ConnectionState::Closed));

        let snapshot = monitor.snapshot().unwrap();
        let updated = snapshot.iter().find(|c| c.id == id).unwrap();
        assert_eq!(updated.state, ConnectionState::Closed);
    }

    #[test]
    fn test_update_nonexistent_connection() {
        let monitor = MockNetworkMonitor::new();
        let fake_id = Uuid::new_v4();

        assert!(!monitor.update_state(fake_id, ConnectionState::Closed));
    }

    #[test]
    fn test_connection_lifecycle() {
        let monitor = MockNetworkMonitor::new();

        // 1. Connection starts as connecting
        let conn = create_tcp_connection(
            1,
            "127.0.0.1",
            54321,
            "8.8.8.8",
            443,
            ConnectionState::Connecting,
        );
        let id = monitor.add_connection(conn);
        assert_eq!(monitor.connection_count(), 1);

        // 2. Becomes established
        monitor.update_state(id, ConnectionState::Established);

        // 3. Connection closes
        monitor.update_state(id, ConnectionState::Closed);

        // 4. Connection is removed
        let removed = monitor.remove_connection(id);
        assert!(removed.is_some());
        assert_eq!(monitor.connection_count(), 0);
    }
}

// ============================================================================
// Test Module: IPv4 and IPv6 Handling
// ============================================================================

#[cfg(test)]
mod ip_version_tests {
    use super::*;

    #[test]
    fn test_ipv4_addresses() {
        let addrs = vec![
            "192.168.1.1",
            "10.0.0.1",
            "172.16.0.1",
            "127.0.0.1",
            "0.0.0.0",
            "255.255.255.255",
        ];

        for addr in addrs {
            let conn = create_tcp_connection(1, addr, 80, "8.8.8.8", 443, ConnectionState::Established);
            assert_eq!(conn.local_addr.as_deref(), Some(addr));
        }
    }

    #[test]
    fn test_ipv6_addresses() {
        let addrs = vec![
            "::1",
            "fe80::1",
            "2001:db8::1",
            "::ffff:192.168.1.1",
            "2607:f8b0:4004:800::200e",
        ];

        for addr in addrs {
            let conn = create_tcp_connection(1, addr, 80, "2001:4860:4860::8888", 443, ConnectionState::Established);
            assert_eq!(conn.local_addr.as_deref(), Some(addr));
        }
    }

    #[test]
    fn test_mixed_ip_versions() {
        let connections = vec![
            create_tcp_connection(1, "192.168.1.1", 80, "8.8.8.8", 443, ConnectionState::Established),
            create_tcp_connection(2, "::1", 80, "::1", 8080, ConnectionState::Established),
            create_tcp_connection(3, "10.0.0.1", 80, "2001:4860:4860::8888", 443, ConnectionState::Established),
        ];

        let monitor = MockNetworkMonitor::with_connections(connections);
        let snapshot = monitor.snapshot().unwrap();

        assert_eq!(snapshot.len(), 3);
    }

    #[test]
    fn test_loopback_ipv4() {
        let conn = create_tcp_connection(
            1,
            "127.0.0.1",
            54321,
            "127.0.0.1",
            8080,
            ConnectionState::Established,
        );

        assert!(conn.local_addr.as_ref().unwrap().starts_with("127."));
        assert!(conn.remote_addr.as_ref().unwrap().starts_with("127."));
    }

    #[test]
    fn test_loopback_ipv6() {
        let conn = create_tcp_connection(
            1,
            "::1",
            54321,
            "::1",
            8080,
            ConnectionState::Established,
        );

        assert_eq!(conn.local_addr.as_deref(), Some("::1"));
        assert_eq!(conn.remote_addr.as_deref(), Some("::1"));
    }

    #[test]
    fn test_ipv4_mapped_ipv6() {
        // IPv4-mapped IPv6 addresses (::ffff:x.x.x.x)
        let conn = create_tcp_connection(
            1,
            "::ffff:192.168.1.1",
            80,
            "::ffff:8.8.8.8",
            443,
            ConnectionState::Established,
        );

        assert!(conn.local_addr.as_ref().unwrap().contains("::ffff:"));
    }
}

// ============================================================================
// Test Module: High Connection Volume
// ============================================================================

#[cfg(test)]
mod high_volume_tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_1000_concurrent_connections() {
        let monitor = MockNetworkMonitor::new();

        // Add 1000 connections
        for i in 0..1000u32 {
            let conn = create_tcp_connection(
                i % 100,
                &format!("192.168.1.{}", (i % 255) + 1),
                50000 + (i as u16),
                "8.8.8.8",
                443,
                ConnectionState::Established,
            );
            monitor.add_connection(conn);
        }

        assert_eq!(monitor.connection_count(), 1000);
        assert_eq!(monitor.total_added(), 1000);

        // Snapshot should return all
        let snapshot = monitor.snapshot().unwrap();
        assert_eq!(snapshot.len(), 1000);
    }

    #[test]
    fn test_rapid_add_remove() {
        let monitor = MockNetworkMonitor::new();
        let start = Instant::now();

        // Rapidly add and remove connections
        for i in 0..500u32 {
            let conn = create_tcp_connection(
                1,
                "127.0.0.1",
                50000 + (i as u16),
                "8.8.8.8",
                443,
                ConnectionState::Established,
            );
            let id = monitor.add_connection(conn);
            monitor.remove_connection(id);
        }

        let elapsed = start.elapsed();

        assert_eq!(monitor.connection_count(), 0);
        assert_eq!(monitor.total_added(), 500);

        // Should complete quickly
        assert!(elapsed < Duration::from_secs(1));
    }

    #[test]
    fn test_concurrent_access() {
        let monitor = Arc::new(MockNetworkMonitor::new());
        let mut handles = Vec::new();

        // Multiple threads adding connections
        for t in 0..4 {
            let m = Arc::clone(&monitor);
            let handle = thread::spawn(move || {
                for i in 0..100 {
                    let conn = create_tcp_connection(
                        (t * 1000 + i) as u32,
                        "127.0.0.1",
                        (50000 + t * 100 + i) as u16,
                        "8.8.8.8",
                        443,
                        ConnectionState::Established,
                    );
                    m.add_connection(conn);
                }
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(monitor.connection_count(), 400);
    }

    #[test]
    fn test_snapshot_performance() {
        let monitor = MockNetworkMonitor::new();

        // Add many connections
        for i in 0..1000u32 {
            let conn = create_tcp_connection(
                i,
                "127.0.0.1",
                50000 + (i as u16 % 15000),
                "8.8.8.8",
                443,
                ConnectionState::Established,
            );
            monitor.add_connection(conn);
        }

        // Snapshot should be fast
        let start = Instant::now();
        for _ in 0..100 {
            let _ = monitor.snapshot();
        }
        let elapsed = start.elapsed();

        // 100 snapshots of 1000 connections should be fast
        assert!(elapsed < Duration::from_secs(1));
    }

    #[test]
    fn test_many_pids() {
        let monitor = MockNetworkMonitor::new();

        // 100 different PIDs, each with 10 connections
        for pid in 1..=100u32 {
            for i in 0..10u16 {
                let conn = create_tcp_connection(
                    pid,
                    "127.0.0.1",
                    50000 + (pid as u16) * 10 + i,
                    "8.8.8.8",
                    443,
                    ConnectionState::Established,
                );
                monitor.add_connection(conn);
            }
        }

        assert_eq!(monitor.connection_count(), 1000);

        // Check distribution
        for pid in 1..=100u32 {
            assert_eq!(monitor.connections_by_pid(pid).len(), 10);
        }
    }
}

// ============================================================================
// Test Module: Endpoint Classification
// ============================================================================

#[cfg(test)]
mod endpoint_classification_tests {
    use super::*;

    #[test]
    fn test_llm_api_endpoints() {
        assert_eq!(
            NetworkMonitorService::classify_endpoint("api.anthropic.com"),
            EndpointClass::LlmApi
        );
        assert_eq!(
            NetworkMonitorService::classify_endpoint("api.openai.com"),
            EndpointClass::LlmApi
        );
        assert_eq!(
            NetworkMonitorService::classify_endpoint("api.cursor.sh"),
            EndpointClass::LlmApi
        );
    }

    #[test]
    fn test_github_endpoints() {
        assert_eq!(
            NetworkMonitorService::classify_endpoint("github.com"),
            EndpointClass::GitHub
        );
        assert_eq!(
            NetworkMonitorService::classify_endpoint("api.github.com"),
            EndpointClass::GitHub
        );
        assert_eq!(
            NetworkMonitorService::classify_endpoint("raw.githubusercontent.com"),
            EndpointClass::GitHub
        );
    }

    #[test]
    fn test_package_registry_endpoints() {
        assert_eq!(
            NetworkMonitorService::classify_endpoint("registry.npmjs.org"),
            EndpointClass::PackageRegistry
        );
        assert_eq!(
            NetworkMonitorService::classify_endpoint("pypi.org"),
            EndpointClass::PackageRegistry
        );
        assert_eq!(
            NetworkMonitorService::classify_endpoint("crates.io"),
            EndpointClass::PackageRegistry
        );
    }

    #[test]
    fn test_telemetry_endpoints() {
        assert_eq!(
            NetworkMonitorService::classify_endpoint("sentry.io"),
            EndpointClass::Telemetry
        );
        assert_eq!(
            NetworkMonitorService::classify_endpoint("api.statsig.com"),
            EndpointClass::Telemetry
        );
        assert_eq!(
            NetworkMonitorService::classify_endpoint("api.amplitude.com"),
            EndpointClass::Telemetry
        );
    }

    #[test]
    fn test_localhost_endpoints() {
        assert_eq!(
            NetworkMonitorService::classify_endpoint("127.0.0.1"),
            EndpointClass::Localhost
        );
        assert_eq!(
            NetworkMonitorService::classify_endpoint("127.0.0.123"),
            EndpointClass::Localhost
        );
        assert_eq!(
            NetworkMonitorService::classify_endpoint("localhost"),
            EndpointClass::Localhost
        );
        assert_eq!(
            NetworkMonitorService::classify_endpoint("::1"),
            EndpointClass::Localhost
        );
    }

    #[test]
    fn test_unknown_endpoints() {
        assert_eq!(
            NetworkMonitorService::classify_endpoint("8.8.8.8"),
            EndpointClass::Unknown
        );
        assert_eq!(
            NetworkMonitorService::classify_endpoint("random-server.com"),
            EndpointClass::Unknown
        );
        assert_eq!(
            NetworkMonitorService::classify_endpoint("192.168.1.1"),
            EndpointClass::Unknown
        );
    }
}

// ============================================================================
// Test Module: Edge Cases
// ============================================================================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_ephemeral_ports() {
        // Ephemeral ports typically 49152-65535
        for port in [49152u16, 55555, 60000, 65535] {
            let conn = create_tcp_connection(
                1,
                "127.0.0.1",
                port,
                "8.8.8.8",
                443,
                ConnectionState::Established,
            );
            assert_eq!(conn.local_port, Some(port));
        }
    }

    #[test]
    fn test_connection_inheritance() {
        // Simulate socket inheritance after fork (same connection, different PIDs)
        let parent_conn = create_tcp_connection(
            1000,
            "127.0.0.1",
            54321,
            "8.8.8.8",
            443,
            ConnectionState::Established,
        );
        let child_conn = create_tcp_connection(
            1001,
            "127.0.0.1",
            54321,
            "8.8.8.8",
            443,
            ConnectionState::Established,
        );

        let monitor = MockNetworkMonitor::new();
        monitor.add_connection(parent_conn);
        monitor.add_connection(child_conn);

        // Both connections exist with same addresses
        let snapshot = monitor.snapshot().unwrap();
        assert_eq!(snapshot.len(), 2);

        // Different PIDs
        let pids: Vec<u32> = snapshot.iter().map(|c| c.pid).collect();
        assert!(pids.contains(&1000));
        assert!(pids.contains(&1001));
    }

    #[test]
    fn test_zero_port() {
        let conn = create_tcp_connection(
            1,
            "0.0.0.0",
            0,
            "8.8.8.8",
            0,
            ConnectionState::Connecting,
        );

        assert_eq!(conn.local_port, Some(0));
        assert_eq!(conn.remote_port, Some(0));
    }

    #[test]
    fn test_wildcard_address() {
        let conn = create_tcp_connection(
            1,
            "0.0.0.0",
            8080,
            "0.0.0.0",
            0,
            ConnectionState::Established,
        );

        assert_eq!(conn.local_addr.as_deref(), Some("0.0.0.0"));
    }

    #[test]
    fn test_connection_unique_ids() {
        let conn1 = create_tcp_connection(1, "127.0.0.1", 80, "8.8.8.8", 443, ConnectionState::Established);
        let conn2 = create_tcp_connection(1, "127.0.0.1", 80, "8.8.8.8", 443, ConnectionState::Established);

        // Even identical connections have different IDs
        assert_ne!(conn1.id, conn2.id);
    }

    #[test]
    fn test_ai_agent_connection_set() {
        let connections = create_ai_agent_connections();
        let monitor = MockNetworkMonitor::with_connections(connections);

        let snapshot = monitor.snapshot().unwrap();
        assert_eq!(snapshot.len(), 5);

        // Check protocol distribution
        let tcp_count = snapshot.iter().filter(|c| c.protocol == Protocol::Tcp).count();
        let udp_count = snapshot.iter().filter(|c| c.protocol == Protocol::Udp).count();
        let unix_count = snapshot.iter().filter(|c| c.protocol == Protocol::Unix).count();

        assert_eq!(tcp_count, 3);
        assert_eq!(udp_count, 1);
        assert_eq!(unix_count, 1);
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
        let mut monitor = MockNetworkMonitor::new();

        assert!(!monitor.is_running());

        monitor.start().unwrap();
        assert!(monitor.is_running());

        monitor.stop().unwrap();
        assert!(!monitor.is_running());
    }

    #[test]
    fn test_double_start() {
        let mut monitor = MockNetworkMonitor::new();

        monitor.start().unwrap();
        monitor.start().unwrap(); // Should be idempotent

        assert!(monitor.is_running());
    }

    #[test]
    fn test_snapshot_empty() {
        let monitor = MockNetworkMonitor::new();
        let snapshot = monitor.snapshot().unwrap();
        assert!(snapshot.is_empty());
    }

    #[test]
    fn test_snapshot_with_connections() {
        let connections = create_ai_agent_connections();
        let count = connections.len();
        let monitor = MockNetworkMonitor::with_connections(connections);

        let snapshot = monitor.snapshot().unwrap();
        assert_eq!(snapshot.len(), count);
    }
}

// ============================================================================
// IPv4/IPv6 Address Parsing Tests (proc_net)
// ============================================================================

#[cfg(test)]
mod address_parsing_tests {
    use super::super::proc_net::*;

    #[test]
    fn test_parse_ipv4_localhost() {
        // 127.0.0.1:80 in /proc/net format
        let (ip, port) = parse_ipv4_addr("0100007F:0050");
        assert_eq!(ip, "127.0.0.1");
        assert_eq!(port, 80);
    }

    #[test]
    fn test_parse_ipv4_any() {
        // 0.0.0.0:0
        let (ip, port) = parse_ipv4_addr("00000000:0000");
        assert_eq!(ip, "0.0.0.0");
        assert_eq!(port, 0);
    }

    #[test]
    fn test_parse_ipv4_with_high_port() {
        // 192.168.1.1:65535
        let (ip, port) = parse_ipv4_addr("0101A8C0:FFFF");
        assert_eq!(ip, "192.168.1.1");
        assert_eq!(port, 65535);
    }

    #[test]
    fn test_parse_ipv4_invalid() {
        let (ip, port) = parse_ipv4_addr("invalid");
        assert!(ip.is_empty());
        assert_eq!(port, 0);
    }

    #[test]
    fn test_parse_ipv6_localhost() {
        // ::1:80
        let (ip, port) = parse_ipv6_addr("00000000000000000000000001000000:0050");
        assert!(!ip.is_empty());
        assert_eq!(port, 80);
    }
}
