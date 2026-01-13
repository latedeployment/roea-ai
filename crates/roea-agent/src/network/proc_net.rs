//! Linux /proc/net based network monitor
//!
//! Parses /proc/net/tcp, /proc/net/udp, and /proc/net/unix for connection info.

use std::collections::HashMap;
use std::fs;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::Utc;
use roea_common::{ConnectionInfo, ConnectionState, PlatformError, PlatformResult, Protocol};

use super::NetworkMonitorBackend;

/// Network monitor using /proc/net (Linux) or sysinfo fallback
pub struct ProcNetMonitor {
    running: AtomicBool,
    /// Cache of inode to PID mappings
    inode_pid_cache: HashMap<u64, u32>,
}

impl ProcNetMonitor {
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            inode_pid_cache: HashMap::new(),
        }
    }

    /// Build inode to PID mapping from /proc/*/fd
    fn build_inode_pid_map(&mut self) -> PlatformResult<()> {
        self.inode_pid_cache.clear();

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

            // Skip non-numeric directories
            if !name_str.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            let pid: u32 = match name_str.parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let fd_path = entry.path().join("fd");
            if let Ok(fd_dir) = fs::read_dir(&fd_path) {
                for fd_entry in fd_dir.flatten() {
                    if let Ok(link) = fs::read_link(fd_entry.path()) {
                        let link_str = link.to_string_lossy();
                        // socket:[12345] format
                        if link_str.starts_with("socket:[") {
                            if let Some(inode_str) =
                                link_str.strip_prefix("socket:[").and_then(|s| s.strip_suffix(']'))
                            {
                                if let Ok(inode) = inode_str.parse::<u64>() {
                                    self.inode_pid_cache.insert(inode, pid);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse /proc/net/tcp or /proc/net/tcp6
    fn parse_tcp_file(&self, path: &Path, ipv6: bool) -> Vec<ConnectionInfo> {
        let mut connections = Vec::new();

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return connections,
        };

        for line in content.lines().skip(1) {
            // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 10 {
                continue;
            }

            // Parse local address
            let (local_addr, local_port) = if ipv6 {
                parse_ipv6_addr(parts[1])
            } else {
                parse_ipv4_addr(parts[1])
            };

            // Parse remote address
            let (remote_addr, remote_port) = if ipv6 {
                parse_ipv6_addr(parts[2])
            } else {
                parse_ipv4_addr(parts[2])
            };

            // Parse state
            let state_hex = u8::from_str_radix(parts[3], 16).unwrap_or(0);
            let state = match state_hex {
                1 => ConnectionState::Established,
                2 => ConnectionState::Connecting, // SYN_SENT
                7 => ConnectionState::Closed,     // CLOSE
                _ => ConnectionState::Connecting,
            };

            // Parse inode
            let inode: u64 = parts[9].parse().unwrap_or(0);

            // Get PID from inode
            let pid = self.inode_pid_cache.get(&inode).copied().unwrap_or(0);

            if pid > 0 || !remote_addr.is_empty() {
                let mut conn = ConnectionInfo::new(pid, Protocol::Tcp);
                conn.local_addr = Some(local_addr);
                conn.local_port = Some(local_port);
                conn.remote_addr = if remote_addr.is_empty() || remote_addr == "0.0.0.0" {
                    None
                } else {
                    Some(remote_addr)
                };
                conn.remote_port = if remote_port == 0 {
                    None
                } else {
                    Some(remote_port)
                };
                conn.state = state;
                conn.timestamp = Utc::now();
                connections.push(conn);
            }
        }

        connections
    }

    /// Parse /proc/net/udp or /proc/net/udp6
    fn parse_udp_file(&self, path: &Path, ipv6: bool) -> Vec<ConnectionInfo> {
        let mut connections = Vec::new();

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return connections,
        };

        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 10 {
                continue;
            }

            let (local_addr, local_port) = if ipv6 {
                parse_ipv6_addr(parts[1])
            } else {
                parse_ipv4_addr(parts[1])
            };

            let (remote_addr, remote_port) = if ipv6 {
                parse_ipv6_addr(parts[2])
            } else {
                parse_ipv4_addr(parts[2])
            };

            let inode: u64 = parts[9].parse().unwrap_or(0);
            let pid = self.inode_pid_cache.get(&inode).copied().unwrap_or(0);

            if pid > 0 || !remote_addr.is_empty() {
                let mut conn = ConnectionInfo::new(pid, Protocol::Udp);
                conn.local_addr = Some(local_addr);
                conn.local_port = Some(local_port);
                conn.remote_addr = if remote_addr.is_empty() || remote_addr == "0.0.0.0" {
                    None
                } else {
                    Some(remote_addr)
                };
                conn.remote_port = if remote_port == 0 {
                    None
                } else {
                    Some(remote_port)
                };
                conn.state = ConnectionState::Established;
                conn.timestamp = Utc::now();
                connections.push(conn);
            }
        }

        connections
    }

    /// Parse /proc/net/unix for Unix domain sockets
    fn parse_unix_sockets(&self) -> Vec<ConnectionInfo> {
        let mut connections = Vec::new();

        let content = match fs::read_to_string("/proc/net/unix") {
            Ok(c) => c,
            Err(_) => return connections,
        };

        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 7 {
                continue;
            }

            let inode: u64 = parts[6].parse().unwrap_or(0);
            let pid = self.inode_pid_cache.get(&inode).copied().unwrap_or(0);

            // Socket path (if present)
            let path = if parts.len() > 7 {
                Some(parts[7].to_string())
            } else {
                None
            };

            if pid > 0 {
                let mut conn = ConnectionInfo::new(pid, Protocol::Unix);
                conn.local_addr = path;
                conn.state = ConnectionState::Established;
                conn.timestamp = Utc::now();
                connections.push(conn);
            }
        }

        connections
    }
}

impl Default for ProcNetMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkMonitorBackend for ProcNetMonitor {
    fn start(&mut self) -> PlatformResult<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        // Check if we're on Linux
        #[cfg(not(target_os = "linux"))]
        {
            return Err(PlatformError::NotSupported(
                "/proc/net is only available on Linux".to_string(),
            ));
        }

        self.running.store(true, Ordering::Relaxed);
        tracing::info!("ProcNet network monitor started");
        Ok(())
    }

    fn stop(&mut self) -> PlatformResult<()> {
        self.running.store(false, Ordering::Relaxed);
        tracing::info!("ProcNet network monitor stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn snapshot(&self) -> PlatformResult<Vec<ConnectionInfo>> {
        // Rebuild inode-pid cache
        let mut monitor = Self::new();
        monitor.build_inode_pid_map()?;

        let mut connections = Vec::new();

        // TCP connections
        connections.extend(monitor.parse_tcp_file(Path::new("/proc/net/tcp"), false));
        connections.extend(monitor.parse_tcp_file(Path::new("/proc/net/tcp6"), true));

        // UDP connections
        connections.extend(monitor.parse_udp_file(Path::new("/proc/net/udp"), false));
        connections.extend(monitor.parse_udp_file(Path::new("/proc/net/udp6"), true));

        // Unix sockets
        connections.extend(monitor.parse_unix_sockets());

        Ok(connections)
    }
}

/// Parse IPv4 address in hex format (e.g., "0100007F:0050")
fn parse_ipv4_addr(addr: &str) -> (String, u16) {
    let parts: Vec<&str> = addr.split(':').collect();
    if parts.len() != 2 {
        return (String::new(), 0);
    }

    let ip_hex = parts[0];
    let port_hex = parts[1];

    // Parse IP (in reverse byte order on little-endian)
    let ip = if ip_hex.len() == 8 {
        let bytes: [u8; 4] = [
            u8::from_str_radix(&ip_hex[6..8], 16).unwrap_or(0),
            u8::from_str_radix(&ip_hex[4..6], 16).unwrap_or(0),
            u8::from_str_radix(&ip_hex[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&ip_hex[0..2], 16).unwrap_or(0),
        ];
        Ipv4Addr::from(bytes).to_string()
    } else {
        String::new()
    };

    let port = u16::from_str_radix(port_hex, 16).unwrap_or(0);

    (ip, port)
}

/// Parse IPv6 address in hex format
fn parse_ipv6_addr(addr: &str) -> (String, u16) {
    let parts: Vec<&str> = addr.split(':').collect();
    if parts.len() != 2 {
        return (String::new(), 0);
    }

    let ip_hex = parts[0];
    let port_hex = parts[1];

    // Parse IPv6 (32 hex chars)
    let ip = if ip_hex.len() == 32 {
        let mut bytes = [0u8; 16];
        for i in 0..16 {
            bytes[i] = u8::from_str_radix(&ip_hex[i * 2..i * 2 + 2], 16).unwrap_or(0);
        }
        // Swap byte order for each 4-byte group (Linux stores in host byte order)
        for chunk in bytes.chunks_exact_mut(4) {
            chunk.reverse();
        }
        Ipv6Addr::from(bytes).to_string()
    } else {
        String::new()
    };

    let port = u16::from_str_radix(port_hex, 16).unwrap_or(0);

    (ip, port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ipv4_addr() {
        // 127.0.0.1:80
        let (ip, port) = parse_ipv4_addr("0100007F:0050");
        assert_eq!(ip, "127.0.0.1");
        assert_eq!(port, 80);
    }
}
