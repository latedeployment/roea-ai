// SPDX-License-Identifier: GPL-2.0 OR BSD-3-Clause
// roea-ai System Monitor BPF Program
//
// This BPF program hooks into process, network, and file events to provide
// real-time visibility into AI agent behavior with minimal overhead.
//
// Tracepoints:
// - sched_process_exec: Process execution
// - sched_process_exit: Process termination
// - sys_enter_connect: Network connections (TCP/UDP/Unix)
// - sys_enter_openat: File opens

#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

// Maximum lengths
#define MAX_COMM_LEN 256
#define MAX_FILENAME_LEN 256
#define MAX_PATH_LEN 256

// Event types
#define EVENT_PROCESS_EXEC 1
#define EVENT_PROCESS_EXIT 2
#define EVENT_NETWORK_CONNECT 3
#define EVENT_FILE_OPEN 4

// Address families (from socket.h)
#define AF_UNIX 1
#define AF_INET 2
#define AF_INET6 10

// Process event structure
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

// Network event structure
struct network_event {
    u32 event_type;
    u32 pid;
    u32 uid;
    u64 timestamp_ns;
    char comm[MAX_COMM_LEN];
    u16 family;         // AF_INET, AF_INET6, AF_UNIX
    u16 port;           // Remote port (network byte order)
    u32 addr_v4;        // IPv4 address (network byte order)
    u8 addr_v6[16];     // IPv6 address
};

// File event structure
struct file_event {
    u32 event_type;
    u32 pid;
    u32 uid;
    u64 timestamp_ns;
    char comm[MAX_COMM_LEN];
    char path[MAX_PATH_LEN];
    int flags;          // Open flags (O_RDONLY, O_WRONLY, etc.)
    int dirfd;          // Directory fd for relative paths
};

// Ring buffers for each event type
struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 256 * 1024);
} events SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 128 * 1024);
} network_events SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 128 * 1024);
} file_events SEC(".maps");

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

// Hook into network connections (sys_enter_connect tracepoint)
// Captures TCP/UDP/Unix socket connect() calls
SEC("tp/syscalls/sys_enter_connect")
int handle_connect(struct trace_event_raw_sys_enter *ctx)
{
    struct network_event *event;
    struct sockaddr *addr;
    u64 id;
    u32 pid;
    u16 family;

    // Get socket address from syscall args
    // connect(int sockfd, const struct sockaddr *addr, socklen_t addrlen)
    addr = (struct sockaddr *)ctx->args[1];
    if (!addr) {
        return 0;
    }

    // Read address family
    bpf_probe_read_user(&family, sizeof(family), &addr->sa_family);

    // Only track AF_INET, AF_INET6, AF_UNIX
    if (family != AF_INET && family != AF_INET6 && family != AF_UNIX) {
        return 0;
    }

    // Reserve space in ring buffer
    event = bpf_ringbuf_reserve(&network_events, sizeof(*event), 0);
    if (!event) {
        return 0;
    }

    // Get process info
    id = bpf_get_current_pid_tgid();
    pid = id >> 32;

    // Fill event data
    event->event_type = EVENT_NETWORK_CONNECT;
    event->pid = pid;
    event->uid = bpf_get_current_uid_gid() & 0xFFFFFFFF;
    event->timestamp_ns = bpf_ktime_get_ns();
    event->family = family;

    // Get command name
    bpf_get_current_comm(&event->comm, sizeof(event->comm));

    // Initialize address fields
    event->port = 0;
    event->addr_v4 = 0;
    __builtin_memset(event->addr_v6, 0, sizeof(event->addr_v6));

    // Read address based on family
    if (family == AF_INET) {
        struct sockaddr_in addr_in;
        bpf_probe_read_user(&addr_in, sizeof(addr_in), addr);
        event->port = addr_in.sin_port;
        event->addr_v4 = addr_in.sin_addr.s_addr;
    } else if (family == AF_INET6) {
        struct sockaddr_in6 addr_in6;
        bpf_probe_read_user(&addr_in6, sizeof(addr_in6), addr);
        event->port = addr_in6.sin6_port;
        bpf_probe_read_user(&event->addr_v6, sizeof(event->addr_v6),
                            &addr_in6.sin6_addr);
    }
    // For AF_UNIX, we don't capture the path (would need more complex handling)

    // Submit event
    bpf_ringbuf_submit(event, 0);

    return 0;
}

// Hook into file opens (sys_enter_openat tracepoint)
// Captures openat() and open() syscalls
SEC("tp/syscalls/sys_enter_openat")
int handle_openat(struct trace_event_raw_sys_enter *ctx)
{
    struct file_event *event;
    const char *pathname;
    u64 id;
    u32 pid;

    // Reserve space in ring buffer
    event = bpf_ringbuf_reserve(&file_events, sizeof(*event), 0);
    if (!event) {
        return 0;
    }

    // Get process info
    id = bpf_get_current_pid_tgid();
    pid = id >> 32;

    // Fill event data
    event->event_type = EVENT_FILE_OPEN;
    event->pid = pid;
    event->uid = bpf_get_current_uid_gid() & 0xFFFFFFFF;
    event->timestamp_ns = bpf_ktime_get_ns();

    // Get command name
    bpf_get_current_comm(&event->comm, sizeof(event->comm));

    // openat(int dirfd, const char *pathname, int flags, ...)
    event->dirfd = (int)ctx->args[0];
    pathname = (const char *)ctx->args[1];
    event->flags = (int)ctx->args[2];

    // Read pathname (may fail if invalid pointer, that's ok)
    if (pathname) {
        bpf_probe_read_user_str(&event->path, sizeof(event->path), pathname);
    } else {
        event->path[0] = '\0';
    }

    // Submit event
    bpf_ringbuf_submit(event, 0);

    return 0;
}

// License declaration required for BPF programs
char LICENSE[] SEC("license") = "Dual BSD/GPL";
