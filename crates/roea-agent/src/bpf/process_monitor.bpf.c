// SPDX-License-Identifier: GPL-2.0 OR BSD-3-Clause
// roea-ai Process Monitor BPF Program
//
// This BPF program hooks into process lifecycle events to provide
// real-time visibility into AI agent process spawning and execution.

#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

// Maximum command line length to capture
#define MAX_COMM_LEN 256
#define MAX_FILENAME_LEN 256

// Event types
#define EVENT_PROCESS_EXEC 1
#define EVENT_PROCESS_EXIT 2

// Process event structure sent to userspace
struct process_event {
    u32 event_type;
    u32 pid;
    u32 ppid;
    u32 uid;
    u32 gid;
    u64 timestamp_ns;
    char comm[MAX_COMM_LEN];
    char filename[MAX_FILENAME_LEN];
    int exit_code;
};

// Ring buffer for sending events to userspace
struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 256 * 1024);
} events SEC(".maps");

// Get current task's parent PID
static __always_inline u32 get_ppid(void)
{
    struct task_struct *task = (struct task_struct *)bpf_get_current_task();
    struct task_struct *parent;
    u32 ppid = 0;

    parent = BPF_CORE_READ(task, real_parent);
    if (parent) {
        ppid = BPF_CORE_READ(parent, tgid);
    }

    return ppid;
}

// Hook into process execution (sched_process_exec tracepoint)
SEC("tp/sched/sched_process_exec")
int handle_exec(struct trace_event_raw_sched_process_exec *ctx)
{
    struct process_event *event;
    struct task_struct *task;
    u64 id;
    u32 pid, uid, gid;

    // Reserve space in ring buffer
    event = bpf_ringbuf_reserve(&events, sizeof(*event), 0);
    if (!event) {
        return 0;
    }

    // Get process info
    id = bpf_get_current_pid_tgid();
    pid = id >> 32;
    uid = bpf_get_current_uid_gid() & 0xFFFFFFFF;
    gid = bpf_get_current_uid_gid() >> 32;

    // Fill event data
    event->event_type = EVENT_PROCESS_EXEC;
    event->pid = pid;
    event->ppid = get_ppid();
    event->uid = uid;
    event->gid = gid;
    event->timestamp_ns = bpf_ktime_get_ns();
    event->exit_code = 0;

    // Get command name
    bpf_get_current_comm(&event->comm, sizeof(event->comm));

    // Get filename from context (the executed file path)
    unsigned int filename_loc = ctx->__data_loc_filename & 0xFFFF;
    bpf_probe_read_str(&event->filename, sizeof(event->filename),
                       (void *)ctx + filename_loc);

    // Submit event
    bpf_ringbuf_submit(event, 0);

    return 0;
}

// Hook into process exit (sched_process_exit tracepoint)
SEC("tp/sched/sched_process_exit")
int handle_exit(struct trace_event_raw_sched_process_template *ctx)
{
    struct process_event *event;
    struct task_struct *task;
    u64 id;
    u32 pid;

    // Reserve space in ring buffer
    event = bpf_ringbuf_reserve(&events, sizeof(*event), 0);
    if (!event) {
        return 0;
    }

    // Get process info
    id = bpf_get_current_pid_tgid();
    pid = id >> 32;

    // Fill event data
    event->event_type = EVENT_PROCESS_EXIT;
    event->pid = pid;
    event->ppid = get_ppid();
    event->uid = bpf_get_current_uid_gid() & 0xFFFFFFFF;
    event->gid = bpf_get_current_uid_gid() >> 32;
    event->timestamp_ns = bpf_ktime_get_ns();

    // Get command name
    bpf_get_current_comm(&event->comm, sizeof(event->comm));

    // Clear filename for exit events
    event->filename[0] = '\0';

    // Get exit code from task_struct
    task = (struct task_struct *)bpf_get_current_task();
    event->exit_code = BPF_CORE_READ(task, exit_code);

    // Submit event
    bpf_ringbuf_submit(event, 0);

    return 0;
}

// License declaration required for BPF programs
char LICENSE[] SEC("license") = "Dual BSD/GPL";
