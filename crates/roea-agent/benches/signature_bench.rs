//! Agent Signature Matching Benchmarks
//!
//! Measures performance of agent detection and signature matching.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use regex::Regex;
use std::collections::HashMap;

use roea_common::{AgentSignature, ProcessMatcher, SignatureMatcher, ProcessInfo, ProcessState};

/// Generate mock process data for benchmarking
fn generate_processes(count: usize, agent_fraction: f32) -> Vec<ProcessInfo> {
    let agent_names = vec!["claude", "cursor", "aider", "windsurf", "code", "copilot"];
    let normal_names = vec!["bash", "node", "python", "git", "cargo", "npm", "rust-analyzer"];

    (0..count)
        .map(|i| {
            let is_agent = (i as f32 / count as f32) < agent_fraction;
            let name = if is_agent {
                agent_names[i % agent_names.len()].to_string()
            } else {
                normal_names[i % normal_names.len()].to_string()
            };

            ProcessInfo {
                id: uuid::Uuid::new_v4().to_string(),
                pid: (1000 + i) as u32,
                ppid: if i == 0 { 1 } else { (1000 + i / 3) as u32 },
                name: name.clone(),
                exe_path: Some(format!("/usr/bin/{}", name)),
                cmdline: Some(format!("{} --some-arg", name)),
                cwd: Some("/home/user".to_string()),
                user: Some("user".to_string()),
                start_time: chrono::Utc::now(),
                end_time: None,
                state: ProcessState::Running,
                cpu_usage: Some(5.0),
                memory_bytes: Some(1024 * 1024),
                agent_type: None,
            }
        })
        .collect()
}

/// Create default signature matcher
fn create_signature_matcher() -> SignatureMatcher {
    let mut matcher = SignatureMatcher::new();

    // Add Claude Code signature
    matcher.add_signature(AgentSignature {
        name: "claude-code".to_string(),
        display_name: "Claude Code".to_string(),
        icon: None,
        process_matchers: vec![
            ProcessMatcher {
                name: Some("claude".to_string()),
                name_regex: None,
                exe_path: None,
                cmdline_contains: None,
            },
        ],
        expected_endpoints: vec!["api.anthropic.com".to_string()],
        child_process_tracking: true,
        parent_hints: vec![],
    });

    // Add Cursor signature
    matcher.add_signature(AgentSignature {
        name: "cursor".to_string(),
        display_name: "Cursor".to_string(),
        icon: None,
        process_matchers: vec![
            ProcessMatcher {
                name: Some("cursor".to_string()),
                name_regex: None,
                exe_path: None,
                cmdline_contains: None,
            },
            ProcessMatcher {
                name: None,
                name_regex: Some("Cursor.*Helper".to_string()),
                exe_path: None,
                cmdline_contains: None,
            },
        ],
        expected_endpoints: vec!["api.cursor.sh".to_string(), "api.openai.com".to_string()],
        child_process_tracking: true,
        parent_hints: vec![],
    });

    // Add Aider signature
    matcher.add_signature(AgentSignature {
        name: "aider".to_string(),
        display_name: "Aider".to_string(),
        icon: None,
        process_matchers: vec![
            ProcessMatcher {
                name: Some("aider".to_string()),
                name_regex: None,
                exe_path: None,
                cmdline_contains: None,
            },
        ],
        expected_endpoints: vec!["api.openai.com".to_string()],
        child_process_tracking: true,
        parent_hints: vec![],
    });

    // Add Windsurf signature
    matcher.add_signature(AgentSignature {
        name: "windsurf".to_string(),
        display_name: "Windsurf".to_string(),
        icon: None,
        process_matchers: vec![
            ProcessMatcher {
                name: Some("windsurf".to_string()),
                name_regex: None,
                exe_path: None,
                cmdline_contains: None,
            },
        ],
        expected_endpoints: vec!["api.codeium.com".to_string()],
        child_process_tracking: true,
        parent_hints: vec![],
    });

    // Add Copilot signature
    matcher.add_signature(AgentSignature {
        name: "copilot".to_string(),
        display_name: "GitHub Copilot".to_string(),
        icon: None,
        process_matchers: vec![
            ProcessMatcher {
                name: Some("copilot".to_string()),
                name_regex: None,
                exe_path: None,
                cmdline_contains: Some("copilot".to_string()),
            },
        ],
        expected_endpoints: vec!["copilot-proxy.githubusercontent.com".to_string()],
        child_process_tracking: false,
        parent_hints: vec![],
    });

    matcher
}

/// Simple exact match (baseline)
fn exact_match(name: &str, targets: &[&str]) -> bool {
    targets.iter().any(|t| name == *t)
}

/// Regex-based matching
fn regex_match(name: &str, patterns: &[Regex]) -> bool {
    patterns.iter().any(|p| p.is_match(name))
}

fn signature_matching_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("signature_matching_single");

    let matcher = create_signature_matcher();

    // Agent process
    let agent_process = ProcessInfo {
        id: "1".to_string(),
        pid: 1000,
        ppid: 1,
        name: "claude".to_string(),
        exe_path: Some("/usr/bin/claude".to_string()),
        cmdline: Some("claude code".to_string()),
        cwd: None,
        user: None,
        start_time: chrono::Utc::now(),
        end_time: None,
        state: ProcessState::Running,
        cpu_usage: None,
        memory_bytes: None,
        agent_type: None,
    };

    // Non-agent process
    let normal_process = ProcessInfo {
        id: "2".to_string(),
        pid: 1001,
        ppid: 1,
        name: "bash".to_string(),
        exe_path: Some("/bin/bash".to_string()),
        cmdline: Some("bash".to_string()),
        cwd: None,
        user: None,
        start_time: chrono::Utc::now(),
        end_time: None,
        state: ProcessState::Running,
        cpu_usage: None,
        memory_bytes: None,
        agent_type: None,
    };

    group.bench_function("match_agent", |b| {
        b.iter(|| matcher.match_process(black_box(&agent_process)))
    });

    group.bench_function("match_non_agent", |b| {
        b.iter(|| matcher.match_process(black_box(&normal_process)))
    });

    group.finish();
}

fn signature_matching_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("signature_matching_batch");

    let matcher = create_signature_matcher();

    for size in [100, 500, 1000].iter() {
        // 10% agents
        let processes = generate_processes(*size, 0.1);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("10pct_agents", size),
            &processes,
            |b, procs| {
                b.iter(|| {
                    procs
                        .iter()
                        .filter_map(|p| matcher.match_process(p))
                        .count()
                })
            },
        );
    }

    for size in [100, 500, 1000].iter() {
        // 50% agents
        let processes = generate_processes(*size, 0.5);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("50pct_agents", size),
            &processes,
            |b, procs| {
                b.iter(|| {
                    procs
                        .iter()
                        .filter_map(|p| matcher.match_process(p))
                        .count()
                })
            },
        );
    }

    group.finish();
}

fn exact_vs_regex_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("exact_vs_regex_matching");

    let targets = vec!["claude", "cursor", "aider", "windsurf", "copilot"];
    let target_refs: Vec<&str> = targets.iter().map(|s| s.as_str()).collect();

    let patterns: Vec<Regex> = targets
        .iter()
        .map(|t| Regex::new(&format!("^{}$", t)).unwrap())
        .collect();

    let test_names = vec![
        "claude",
        "bash",
        "cursor",
        "node",
        "aider",
        "python",
        "windsurf",
        "git",
    ];

    group.bench_function("exact_match", |b| {
        b.iter(|| {
            test_names
                .iter()
                .filter(|n| exact_match(n, &target_refs))
                .count()
        })
    });

    group.bench_function("regex_match", |b| {
        b.iter(|| {
            test_names
                .iter()
                .filter(|n| regex_match(n, &patterns))
                .count()
        })
    });

    group.finish();
}

fn signature_lookup_by_name(c: &mut Criterion) {
    let mut group = c.benchmark_group("signature_lookup_by_name");

    let matcher = create_signature_matcher();

    group.bench_function("lookup_existing", |b| {
        b.iter(|| matcher.get_signature(black_box("claude-code")))
    });

    group.bench_function("lookup_nonexistent", |b| {
        b.iter(|| matcher.get_signature(black_box("unknown-agent")))
    });

    group.finish();
}

fn signature_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("signature_iteration");

    let matcher = create_signature_matcher();

    group.bench_function("iterate_all_signatures", |b| {
        b.iter(|| {
            let count: usize = matcher.signatures().count();
            count
        })
    });

    group.bench_function("collect_signature_names", |b| {
        b.iter(|| {
            let names: Vec<&str> = matcher.signatures().map(|s| s.name.as_str()).collect();
            names
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    signature_matching_single,
    signature_matching_batch,
    exact_vs_regex_matching,
    signature_lookup_by_name,
    signature_iteration,
);

criterion_main!(benches);
