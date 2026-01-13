//! Process Monitoring Benchmarks
//!
//! Measures performance of process monitoring operations.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::hint::black_box as hint_black_box;
use std::time::{Duration, Instant};

use roea_common::{ProcessInfo, ProcessState};

/// Generate mock process data for benchmarking
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
            state: if i % 3 == 0 {
                ProcessState::Running
            } else if i % 3 == 1 {
                ProcessState::Sleeping
            } else {
                ProcessState::Stopped
            },
            cpu_usage: Some((i % 100) as f32),
            memory_bytes: Some((i * 1024 * 1024) as u64),
            agent_type: if i % 10 == 0 {
                Some("claude-code".to_string())
            } else {
                None
            },
        })
        .collect()
}

/// Build a process tree from flat list
fn build_process_tree(processes: &[ProcessInfo]) -> HashMap<u32, Vec<u32>> {
    let mut tree: HashMap<u32, Vec<u32>> = HashMap::new();

    for process in processes {
        tree.entry(process.ppid)
            .or_default()
            .push(process.pid);
    }

    tree
}

/// Find all children recursively
fn find_all_children(tree: &HashMap<u32, Vec<u32>>, pid: u32) -> Vec<u32> {
    let mut result = Vec::new();
    let mut stack = vec![pid];

    while let Some(current) = stack.pop() {
        if let Some(children) = tree.get(&current) {
            for &child in children {
                result.push(child);
                stack.push(child);
            }
        }
    }

    result
}

/// Filter processes by agent type
fn filter_by_agent(processes: &[ProcessInfo], agent_type: &str) -> Vec<&ProcessInfo> {
    processes
        .iter()
        .filter(|p| p.agent_type.as_deref() == Some(agent_type))
        .collect()
}

/// Search processes by name (case insensitive)
fn search_by_name<'a>(processes: &'a [ProcessInfo], query: &str) -> Vec<&'a ProcessInfo> {
    let query_lower = query.to_lowercase();
    processes
        .iter()
        .filter(|p| p.name.to_lowercase().contains(&query_lower))
        .collect()
}

fn process_tree_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_tree_construction");

    for size in [10, 50, 100, 500, 1000].iter() {
        let processes = generate_processes(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &processes, |b, procs| {
            b.iter(|| build_process_tree(black_box(procs)))
        });
    }

    group.finish();
}

fn process_child_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_child_lookup");

    for size in [100, 500, 1000].iter() {
        let processes = generate_processes(*size);
        let tree = build_process_tree(&processes);
        let root_pid = 1000_u32;

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &(tree, root_pid),
            |b, (tree, pid)| {
                b.iter(|| find_all_children(black_box(tree), black_box(*pid)))
            },
        );
    }

    group.finish();
}

fn process_filtering(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_filtering");

    for size in [100, 500, 1000, 5000].iter() {
        let processes = generate_processes(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("by_agent_type", size),
            &processes,
            |b, procs| {
                b.iter(|| filter_by_agent(black_box(procs), black_box("claude-code")))
            },
        );
    }

    group.finish();
}

fn process_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_search");

    for size in [100, 500, 1000, 5000].iter() {
        let processes = generate_processes(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &processes, |b, procs| {
            b.iter(|| search_by_name(black_box(procs), black_box("process")))
        });
    }

    group.finish();
}

fn process_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_iteration");

    for size in [100, 1000, 10000].iter() {
        let processes = generate_processes(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &processes, |b, procs| {
            b.iter(|| {
                let mut total_cpu = 0.0_f32;
                let mut total_mem = 0_u64;
                for p in procs {
                    total_cpu += p.cpu_usage.unwrap_or(0.0);
                    total_mem += p.memory_bytes.unwrap_or(0);
                }
                hint_black_box((total_cpu, total_mem))
            })
        });
    }

    group.finish();
}

fn process_pid_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_pid_lookup");

    for size in [100, 1000, 10000].iter() {
        let processes = generate_processes(*size);
        let pid_map: HashMap<u32, &ProcessInfo> =
            processes.iter().map(|p| (p.pid, p)).collect();
        let target_pid = 1500_u32;

        group.bench_with_input(BenchmarkId::from_parameter(size), &pid_map, |b, map| {
            b.iter(|| map.get(black_box(&target_pid)))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    process_tree_construction,
    process_child_lookup,
    process_filtering,
    process_search,
    process_iteration,
    process_pid_lookup,
);

criterion_main!(benches);
