//! Network Monitoring Benchmarks
//!
//! Measures performance of network connection tracking operations.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::{HashMap, HashSet};
use std::hint::black_box as hint_black_box;

use roea_common::{ConnectionInfo, ConnectionState, Protocol};

/// Generate mock connection data for benchmarking
fn generate_connections(count: usize) -> Vec<ConnectionInfo> {
    let protocols = [Protocol::Tcp, Protocol::Udp, Protocol::Unix];
    let states = [
        ConnectionState::Connecting,
        ConnectionState::Established,
        ConnectionState::Closed,
    ];
    let endpoints = [
        "api.anthropic.com",
        "api.openai.com",
        "api.cursor.sh",
        "github.com",
        "registry.npmjs.org",
        "crates.io",
        "127.0.0.1",
    ];

    (0..count)
        .map(|i| {
            let mut conn = ConnectionInfo::new((1000 + i % 100) as u32, protocols[i % 3].clone());
            conn.id = uuid::Uuid::new_v4().to_string();
            conn.local_addr = Some("127.0.0.1".to_string());
            conn.local_port = Some((40000 + i) as u16);
            conn.remote_addr = Some(endpoints[i % endpoints.len()].to_string());
            conn.remote_port = Some(443);
            conn.state = states[i % 3].clone();
            conn.bytes_sent = Some((i * 1024) as u64);
            conn.bytes_received = Some((i * 2048) as u64);
            conn.timestamp = chrono::Utc::now();
            conn
        })
        .collect()
}

/// Group connections by process ID
fn group_by_pid(connections: &[ConnectionInfo]) -> HashMap<u32, Vec<&ConnectionInfo>> {
    let mut groups: HashMap<u32, Vec<&ConnectionInfo>> = HashMap::new();

    for conn in connections {
        groups.entry(conn.pid).or_default().push(conn);
    }

    groups
}

/// Group connections by remote endpoint
fn group_by_endpoint(connections: &[ConnectionInfo]) -> HashMap<String, Vec<&ConnectionInfo>> {
    let mut groups: HashMap<String, Vec<&ConnectionInfo>> = HashMap::new();

    for conn in connections {
        if let Some(ref addr) = conn.remote_addr {
            let key = format!("{}:{}", addr, conn.remote_port.unwrap_or(0));
            groups.entry(key).or_default().push(conn);
        }
    }

    groups
}

/// Filter connections by state
fn filter_by_state(connections: &[ConnectionInfo], state: ConnectionState) -> Vec<&ConnectionInfo> {
    connections.iter().filter(|c| c.state == state).collect()
}

/// Filter connections by protocol
fn filter_by_protocol(connections: &[ConnectionInfo], protocol: &Protocol) -> Vec<&ConnectionInfo> {
    connections
        .iter()
        .filter(|c| &c.protocol == protocol)
        .collect()
}

/// Get unique remote endpoints
fn unique_endpoints(connections: &[ConnectionInfo]) -> HashSet<String> {
    connections
        .iter()
        .filter_map(|c| {
            c.remote_addr.as_ref().map(|addr| {
                format!("{}:{}", addr, c.remote_port.unwrap_or(0))
            })
        })
        .collect()
}

/// Calculate total bandwidth
fn calculate_bandwidth(connections: &[ConnectionInfo]) -> (u64, u64) {
    connections.iter().fold((0, 0), |(sent, recv), conn| {
        (
            sent + conn.bytes_sent.unwrap_or(0),
            recv + conn.bytes_received.unwrap_or(0),
        )
    })
}

/// Parse IPv4 address from hex format (benchmark the parsing)
fn parse_ipv4_hex(addr: &str) -> Option<String> {
    if addr.len() != 8 {
        return None;
    }

    let bytes: Vec<u8> = (0..4)
        .filter_map(|i| u8::from_str_radix(&addr[i * 2..i * 2 + 2], 16).ok())
        .collect();

    if bytes.len() == 4 {
        Some(format!("{}.{}.{}.{}", bytes[3], bytes[2], bytes[1], bytes[0]))
    } else {
        None
    }
}

fn connection_grouping_by_pid(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_grouping_by_pid");

    for size in [100, 500, 1000, 5000].iter() {
        let connections = generate_connections(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &connections,
            |b, conns| {
                b.iter(|| group_by_pid(black_box(conns)))
            },
        );
    }

    group.finish();
}

fn connection_grouping_by_endpoint(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_grouping_by_endpoint");

    for size in [100, 500, 1000, 5000].iter() {
        let connections = generate_connections(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &connections,
            |b, conns| {
                b.iter(|| group_by_endpoint(black_box(conns)))
            },
        );
    }

    group.finish();
}

fn connection_filtering_by_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_filtering_by_state");

    for size in [100, 1000, 10000].iter() {
        let connections = generate_connections(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &connections,
            |b, conns| {
                b.iter(|| filter_by_state(black_box(conns), ConnectionState::Established))
            },
        );
    }

    group.finish();
}

fn connection_filtering_by_protocol(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_filtering_by_protocol");

    for size in [100, 1000, 10000].iter() {
        let connections = generate_connections(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &connections,
            |b, conns| {
                b.iter(|| filter_by_protocol(black_box(conns), &Protocol::Tcp))
            },
        );
    }

    group.finish();
}

fn connection_unique_endpoints(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_unique_endpoints");

    for size in [100, 1000, 5000].iter() {
        let connections = generate_connections(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &connections,
            |b, conns| {
                b.iter(|| unique_endpoints(black_box(conns)))
            },
        );
    }

    group.finish();
}

fn connection_bandwidth_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_bandwidth_calculation");

    for size in [100, 1000, 10000].iter() {
        let connections = generate_connections(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &connections,
            |b, conns| {
                b.iter(|| calculate_bandwidth(black_box(conns)))
            },
        );
    }

    group.finish();
}

fn ipv4_hex_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipv4_hex_parsing");

    let test_addrs = vec![
        "0100007F", // 127.0.0.1
        "C0A80001", // 192.168.0.1
        "08080808", // 8.8.8.8
    ];

    for addr in test_addrs {
        group.bench_with_input(BenchmarkId::from_parameter(addr), &addr, |b, a| {
            b.iter(|| parse_ipv4_hex(black_box(a)))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    connection_grouping_by_pid,
    connection_grouping_by_endpoint,
    connection_filtering_by_state,
    connection_filtering_by_protocol,
    connection_unique_endpoints,
    connection_bandwidth_calculation,
    ipv4_hex_parsing,
);

criterion_main!(benches);
