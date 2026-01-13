//! Storage Layer Benchmarks
//!
//! Measures performance of DuckDB storage operations.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Instant;
use tempfile::TempDir;

use roea_common::{ProcessInfo, ProcessState, ConnectionInfo, ConnectionState, Protocol, FileOpInfo, FileOperation};

/// Generate mock process data
fn generate_processes(count: usize) -> Vec<ProcessInfo> {
    (0..count)
        .map(|i| ProcessInfo {
            id: uuid::Uuid::new_v4().to_string(),
            pid: (1000 + i) as u32,
            ppid: if i == 0 { 1 } else { (1000 + i / 3) as u32 },
            name: format!("process-{}", i),
            exe_path: Some(format!("/usr/bin/process-{}", i)),
            cmdline: Some(format!("process-{} --arg={}", i, i)),
            cwd: Some("/home/user".to_string()),
            user: Some("user".to_string()),
            start_time: chrono::Utc::now(),
            end_time: None,
            state: ProcessState::Running,
            cpu_usage: Some(5.0),
            memory_bytes: Some(1024 * 1024),
            agent_type: if i % 10 == 0 { Some("claude-code".to_string()) } else { None },
        })
        .collect()
}

/// Generate mock connection data
fn generate_connections(count: usize) -> Vec<ConnectionInfo> {
    (0..count)
        .map(|i| {
            let mut conn = ConnectionInfo::new((1000 + i % 100) as u32, Protocol::Tcp);
            conn.id = uuid::Uuid::new_v4().to_string();
            conn.local_addr = Some("127.0.0.1".to_string());
            conn.local_port = Some((40000 + i) as u16);
            conn.remote_addr = Some("api.anthropic.com".to_string());
            conn.remote_port = Some(443);
            conn.state = ConnectionState::Established;
            conn.timestamp = chrono::Utc::now();
            conn
        })
        .collect()
}

/// Generate mock file operation data
fn generate_file_ops(count: usize) -> Vec<FileOpInfo> {
    let ops = [
        FileOperation::Open,
        FileOperation::Read,
        FileOperation::Write,
        FileOperation::Create,
    ];

    (0..count)
        .map(|i| FileOpInfo {
            id: uuid::Uuid::new_v4().to_string(),
            process_id: format!("proc-{}", i % 100),
            pid: (1000 + i % 100) as u32,
            operation: ops[i % ops.len()].clone(),
            path: format!("/home/user/project/file-{}.txt", i),
            timestamp: chrono::Utc::now(),
            success: true,
            error_message: None,
        })
        .collect()
}

/// Serialize process to JSON (simulating storage prep)
fn serialize_process(process: &ProcessInfo) -> String {
    serde_json::to_string(process).unwrap_or_default()
}

/// Deserialize process from JSON
fn deserialize_process(json: &str) -> Option<ProcessInfo> {
    serde_json::from_str(json).ok()
}

/// Batch serialize processes
fn batch_serialize_processes(processes: &[ProcessInfo]) -> Vec<String> {
    processes.iter().map(serialize_process).collect()
}

fn process_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_serialization");

    let process = generate_processes(1).remove(0);

    group.bench_function("single_serialize", |b| {
        b.iter(|| serialize_process(black_box(&process)))
    });

    let json = serialize_process(&process);
    group.bench_function("single_deserialize", |b| {
        b.iter(|| deserialize_process(black_box(&json)))
    });

    group.finish();
}

fn batch_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_serialization");

    for size in [10, 100, 1000].iter() {
        let processes = generate_processes(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &processes,
            |b, procs| {
                b.iter(|| batch_serialize_processes(black_box(procs)))
            },
        );
    }

    group.finish();
}

fn connection_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_serialization");

    for size in [10, 100, 1000].iter() {
        let connections = generate_connections(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &connections,
            |b, conns| {
                b.iter(|| {
                    let serialized: Vec<String> = conns
                        .iter()
                        .map(|c| serde_json::to_string(c).unwrap_or_default())
                        .collect();
                    serialized
                })
            },
        );
    }

    group.finish();
}

fn file_op_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_op_serialization");

    for size in [10, 100, 1000].iter() {
        let file_ops = generate_file_ops(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &file_ops,
            |b, ops| {
                b.iter(|| {
                    let serialized: Vec<String> = ops
                        .iter()
                        .map(|o| serde_json::to_string(o).unwrap_or_default())
                        .collect();
                    serialized
                })
            },
        );
    }

    group.finish();
}

fn data_generation_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_generation_overhead");

    group.bench_function("generate_100_processes", |b| {
        b.iter(|| generate_processes(black_box(100)))
    });

    group.bench_function("generate_100_connections", |b| {
        b.iter(|| generate_connections(black_box(100)))
    });

    group.bench_function("generate_100_file_ops", |b| {
        b.iter(|| generate_file_ops(black_box(100)))
    });

    group.finish();
}

/// Benchmark memory allocation patterns
fn memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    // Pre-allocated vs dynamic Vec
    group.bench_function("preallocated_vec_1000", |b| {
        b.iter(|| {
            let mut v: Vec<ProcessInfo> = Vec::with_capacity(1000);
            for i in 0..1000 {
                v.push(ProcessInfo {
                    id: i.to_string(),
                    pid: i as u32,
                    ppid: 1,
                    name: format!("p{}", i),
                    exe_path: None,
                    cmdline: None,
                    cwd: None,
                    user: None,
                    start_time: chrono::Utc::now(),
                    end_time: None,
                    state: ProcessState::Running,
                    cpu_usage: None,
                    memory_bytes: None,
                    agent_type: None,
                });
            }
            v
        })
    });

    group.bench_function("dynamic_vec_1000", |b| {
        b.iter(|| {
            let mut v: Vec<ProcessInfo> = Vec::new();
            for i in 0..1000 {
                v.push(ProcessInfo {
                    id: i.to_string(),
                    pid: i as u32,
                    ppid: 1,
                    name: format!("p{}", i),
                    exe_path: None,
                    cmdline: None,
                    cwd: None,
                    user: None,
                    start_time: chrono::Utc::now(),
                    end_time: None,
                    state: ProcessState::Running,
                    cpu_usage: None,
                    memory_bytes: None,
                    agent_type: None,
                });
            }
            v
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    process_serialization,
    batch_serialization,
    connection_serialization,
    file_op_serialization,
    data_generation_overhead,
    memory_allocation,
);

criterion_main!(benches);
