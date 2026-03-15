//! Windows network monitor using `netstat -ano`
//!
//! Parses `netstat -ano` output to get TCP/UDP connections with their PIDs.
//! This is the Windows equivalent of the Linux /proc/net approach.
//!
//! The `parse_netstat_output` function is public so it can be unit-tested with
//! captured output on any platform without needing a Windows machine.

use std::sync::atomic::{AtomicBool, Ordering};

use chrono::Utc;
use tuai_common::{ConnectionInfo, ConnectionState, PlatformError, PlatformResult, Protocol};

use super::NetworkMonitorBackend;

/// Network monitor using `netstat -ano` (Windows)
pub struct WindowsNetMonitor {
    running: AtomicBool,
}

impl WindowsNetMonitor {
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
        }
    }

    fn collect() -> PlatformResult<Vec<ConnectionInfo>> {
        let output = std::process::Command::new("netstat")
            .args(["-ano"])
            .output()
            .map_err(|e| PlatformError::CollectionFailed(format!("netstat failed: {}", e)))?;

        if !output.status.success() {
            return Err(PlatformError::CollectionFailed(
                "netstat exited with non-zero status".into(),
            ));
        }

        let text = String::from_utf8_lossy(&output.stdout);
        Ok(parse_netstat_output(&text))
    }
}

impl Default for WindowsNetMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkMonitorBackend for WindowsNetMonitor {
    fn start(&mut self) -> PlatformResult<()> {
        self.running.store(true, Ordering::Relaxed);
        tracing::info!("Windows netstat network monitor started");
        Ok(())
    }

    fn stop(&mut self) -> PlatformResult<()> {
        self.running.store(false, Ordering::Relaxed);
        tracing::info!("Windows netstat network monitor stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn snapshot(&self) -> PlatformResult<Vec<ConnectionInfo>> {
        Self::collect()
    }
}

/// Parse `netstat -ano` output into a list of connections.
///
/// Public so it can be unit-tested on any platform using captured output.
///
/// Expected column layout:
/// ```text
/// TCP    192.168.1.1:52341    93.184.216.34:443    ESTABLISHED    8765
/// UDP    0.0.0.0:5353         *:*                                 1044
/// ```
pub fn parse_netstat_output(output: &str) -> Vec<ConnectionInfo> {
    let mut connections = Vec::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();

        let proto = match parts.first() {
            Some(p) => *p,
            None => continue,
        };

        if proto.eq_ignore_ascii_case("tcp") && parts.len() >= 5 {
            let pid: u32 = match parts[4].parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let (local_addr, local_port) = parse_addr(parts[1]);
            let (remote_addr, remote_port) = parse_addr(parts[2]);
            let state = parse_state(parts[3]);

            let mut conn = ConnectionInfo::new(pid, Protocol::Tcp);
            conn.local_addr = if local_addr.is_empty() { None } else { Some(local_addr) };
            conn.local_port = local_port;
            conn.remote_addr = if remote_addr.is_empty()
                || remote_addr == "0.0.0.0"
                || remote_addr == "::"
            {
                None
            } else {
                Some(remote_addr)
            };
            conn.remote_port = remote_port.filter(|&p| p != 0);
            conn.state = state;
            conn.timestamp = Utc::now();
            connections.push(conn);
        } else if proto.eq_ignore_ascii_case("udp") && parts.len() >= 4 {
            let pid: u32 = match parts[3].parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let (local_addr, local_port) = parse_addr(parts[1]);

            let mut conn = ConnectionInfo::new(pid, Protocol::Udp);
            conn.local_addr = if local_addr.is_empty() { None } else { Some(local_addr) };
            conn.local_port = local_port;
            conn.state = ConnectionState::Established;
            conn.timestamp = Utc::now();
            connections.push(conn);
        }
    }

    connections
}

/// Parse a netstat address string into `(ip, port)`.
///
/// Handles:
/// - IPv4:  `192.168.1.1:8080`
/// - IPv6:  `[::1]:8080` or `[::]:0`
/// - Empty: `*:*`
fn parse_addr(addr: &str) -> (String, Option<u16>) {
    if addr == "*:*" {
        return (String::new(), None);
    }

    // IPv6: [::1]:8080
    if addr.starts_with('[') {
        if let Some(bracket_end) = addr.find(']') {
            let ip = addr[1..bracket_end].to_string();
            let port = addr.get(bracket_end + 2..).and_then(|s| s.parse().ok());
            return (ip, port);
        }
    }

    // IPv4: 1.2.3.4:port
    if let Some(colon_pos) = addr.rfind(':') {
        let ip = addr[..colon_pos].to_string();
        let port = addr[colon_pos + 1..].parse().ok();
        return (ip, port);
    }

    (addr.to_string(), None)
}

/// Map netstat state strings to `ConnectionState`.
fn parse_state(state: &str) -> ConnectionState {
    match state {
        "ESTABLISHED" => ConnectionState::Established,
        "LISTENING" => ConnectionState::Listen,
        "TIME_WAIT" => ConnectionState::TimeWait,
        "CLOSE_WAIT" => ConnectionState::CloseWait,
        "CLOSED" => ConnectionState::Closed,
        "SYN_SENT" | "SYN_RECEIVED" => ConnectionState::Connecting,
        _ => ConnectionState::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Captured from `netstat -ano` on Windows 11
    const SAMPLE: &str = "\
\r\nActive Connections\r\n\
\r\n  Proto  Local Address          Foreign Address        State           PID\r\n\
  TCP    0.0.0.0:135            0.0.0.0:0              LISTENING       1044\r\n\
  TCP    127.0.0.1:50051        0.0.0.0:0              LISTENING       12345\r\n\
  TCP    192.168.1.100:52341    93.184.216.34:443      ESTABLISHED     8765\r\n\
  TCP    192.168.1.100:51000    52.168.112.60:443      TIME_WAIT       0\r\n\
  TCP    192.168.1.100:51001    52.168.112.60:443      CLOSE_WAIT      4321\r\n\
  TCP    [::]:135               [::]:0                 LISTENING       1044\r\n\
  TCP    [::1]:6443             [::]:0                 LISTENING       9900\r\n\
  UDP    0.0.0.0:5353           *:*                                    1044\r\n\
  UDP    127.0.0.1:55000        *:*                                    9876\r\n\
";

    #[test]
    fn test_total_count() {
        let conns = parse_netstat_output(SAMPLE);
        // 7 TCP + 2 UDP
        assert_eq!(conns.len(), 9);
    }

    #[test]
    fn test_tcp_established() {
        let conns = parse_netstat_output(SAMPLE);
        let conn = conns
            .iter()
            .find(|c| c.state == ConnectionState::Established)
            .unwrap();
        assert_eq!(conn.pid, 8765);
        assert_eq!(conn.protocol, Protocol::Tcp);
        assert_eq!(conn.local_addr.as_deref(), Some("192.168.1.100"));
        assert_eq!(conn.local_port, Some(52341));
        assert_eq!(conn.remote_addr.as_deref(), Some("93.184.216.34"));
        assert_eq!(conn.remote_port, Some(443));
    }

    #[test]
    fn test_tcp_listening_no_remote() {
        let conns = parse_netstat_output(SAMPLE);
        let listener = conns.iter().find(|c| c.pid == 12345).unwrap();
        assert_eq!(listener.state, ConnectionState::Listen);
        assert_eq!(listener.local_port, Some(50051));
        // LISTENING remote (0.0.0.0:0) should be suppressed
        assert_eq!(listener.remote_addr, None);
        assert_eq!(listener.remote_port, None);
    }

    #[test]
    fn test_listening_count() {
        let conns = parse_netstat_output(SAMPLE);
        let count = conns.iter().filter(|c| c.state == ConnectionState::Listen).count();
        // 0.0.0.0:135, 127.0.0.1:50051, [::]:135, [::1]:6443
        assert_eq!(count, 4);
    }

    #[test]
    fn test_tcp_time_wait() {
        let conns = parse_netstat_output(SAMPLE);
        let tw = conns.iter().find(|c| c.state == ConnectionState::TimeWait).unwrap();
        assert_eq!(tw.pid, 0);
    }

    #[test]
    fn test_tcp_close_wait() {
        let conns = parse_netstat_output(SAMPLE);
        let cw = conns.iter().find(|c| c.state == ConnectionState::CloseWait).unwrap();
        assert_eq!(cw.pid, 4321);
    }

    #[test]
    fn test_ipv6_listening() {
        let conns = parse_netstat_output(SAMPLE);
        let ipv6 = conns
            .iter()
            .find(|c| c.local_addr.as_deref() == Some("::1"))
            .unwrap();
        assert_eq!(ipv6.local_port, Some(6443));
        assert_eq!(ipv6.state, ConnectionState::Listen);
    }

    #[test]
    fn test_udp_connections() {
        let conns = parse_netstat_output(SAMPLE);
        let udp: Vec<_> = conns.iter().filter(|c| c.protocol == Protocol::Udp).collect();
        assert_eq!(udp.len(), 2);
        assert!(udp.iter().any(|c| c.pid == 9876));
        assert!(udp.iter().any(|c| c.pid == 1044));
    }

    #[test]
    fn test_parse_addr_ipv4() {
        let (ip, port) = parse_addr("192.168.1.1:8080");
        assert_eq!(ip, "192.168.1.1");
        assert_eq!(port, Some(8080));
    }

    #[test]
    fn test_parse_addr_ipv6() {
        let (ip, port) = parse_addr("[::1]:443");
        assert_eq!(ip, "::1");
        assert_eq!(port, Some(443));
    }

    #[test]
    fn test_parse_addr_ipv6_any() {
        let (ip, port) = parse_addr("[::]:0");
        assert_eq!(ip, "::");
        assert_eq!(port, Some(0));
    }

    #[test]
    fn test_parse_addr_wildcard() {
        let (ip, port) = parse_addr("*:*");
        assert_eq!(ip, "");
        assert_eq!(port, None);
    }

    #[test]
    fn test_empty_output() {
        assert!(parse_netstat_output("").is_empty());
    }

    #[test]
    fn test_header_lines_skipped() {
        let header_only = "Active Connections\r\n\r\n  Proto  Local Address  Foreign Address  State  PID\r\n";
        assert!(parse_netstat_output(header_only).is_empty());
    }
}
