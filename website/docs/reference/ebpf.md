# Linux eBPF Setup

roea-ai uses eBPF (Extended Berkeley Packet Filter) on Linux for high-performance, low-latency kernel-level monitoring.

## Overview

eBPF provides:
- **Sub-millisecond** process detection latency
- **Kernel-level** event capture
- **Minimal overhead** (<1% CPU)
- **No kernel modules** required

## Requirements

### Kernel Version

- **Minimum**: Linux 4.18+
- **Recommended**: Linux 5.8+ (for ring buffer support)

Check your kernel version:
```bash
uname -r
```

### BTF (BPF Type Format)

BTF must be enabled in your kernel. Check:
```bash
ls /sys/kernel/btf/vmlinux
```

If the file exists, BTF is enabled.

### Capabilities

roea-agent needs one of:
- Root privileges (`sudo`)
- `CAP_BPF` capability
- `CAP_SYS_ADMIN` (older kernels)

## Setup

### Option 1: Run as Root

```bash
sudo roea-agent
```

### Option 2: Grant Capabilities

```bash
# Grant BPF capability
sudo setcap cap_bpf+ep /usr/local/bin/roea-agent

# Run without sudo
roea-agent
```

### Option 3: Systemd Service

```ini
# /etc/systemd/system/roea-agent.service
[Unit]
Description=roea-ai Monitoring Agent
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/roea-agent
AmbientCapabilities=CAP_BPF
NoNewPrivileges=true
User=roea
Group=roea

[Install]
WantedBy=multi-user.target
```

## eBPF Programs

roea-agent loads these eBPF programs:

### Process Monitoring

```c
// Tracepoints
sched/sched_process_exec  // Process start
sched/sched_process_exit  // Process exit
```

### Network Monitoring

```c
// Tracepoints
sock/inet_sock_set_state  // Connection state changes

// kprobes (fallback)
tcp_connect
tcp_close
```

### File Monitoring

```c
// Tracepoints
syscalls/sys_enter_openat
syscalls/sys_enter_read
syscalls/sys_enter_write
```

## Configuration

```toml
[monitor.ebpf]
# Enable eBPF monitoring
enabled = true

# Fall back to polling if eBPF fails
fallback_to_polling = true

# Ring buffer size (per CPU)
ring_buffer_size = "16MB"

# Enable network tracepoints
network_tracing = true

# Enable file tracepoints
file_tracing = true

# Debug eBPF programs
debug = false
```

## Verification

Check if eBPF is active:

```bash
# Check roea-agent logs
journalctl -u roea-agent | grep -i ebpf

# Should see:
# INFO eBPF process monitoring available
# INFO Using kernel tracepoints for process events
```

List loaded BPF programs:
```bash
sudo bpftool prog list | grep roea
```

## Troubleshooting

### "BTF not found"

If BTF is not available:

```bash
# Generate vmlinux.h from kernel
bpftool btf dump file /sys/kernel/btf/vmlinux format c > vmlinux.h

# Or install kernel debug symbols
sudo apt install linux-headers-$(uname -r)
```

### "Permission denied"

Check capabilities:
```bash
getcap /usr/local/bin/roea-agent
# Should show: cap_bpf=ep

# If not, set it:
sudo setcap cap_bpf+ep /usr/local/bin/roea-agent
```

### "eBPF program failed to load"

Check kernel lockdown:
```bash
cat /sys/kernel/security/lockdown
# If "confidentiality" or "integrity", eBPF may be restricted
```

Possible solutions:
1. Disable Secure Boot
2. Use `CAP_SYS_ADMIN` instead of `CAP_BPF`
3. Fall back to polling mode

### High CPU Usage

If eBPF causes high CPU:

```toml
[monitor.ebpf]
# Increase ring buffer
ring_buffer_size = "32MB"

# Or disable specific tracing
file_tracing = false
```

## Performance

eBPF vs Polling comparison:

| Metric | eBPF | Polling |
|--------|------|---------|
| Detection latency | <1ms | ~1000ms |
| CPU overhead | <0.5% | 1-2% |
| Missed events | None | Possible |
| Kernel version | 4.18+ | Any |

## Fallback Mode

If eBPF fails, roea-agent falls back to polling:

```
WARN eBPF not available, falling back to sysinfo polling
INFO Process monitoring using 1000ms polling interval
```

To force polling mode:
```toml
[monitor.ebpf]
enabled = false
```

## Security Considerations

eBPF programs run in the kernel. roea-agent's eBPF code:
- Is verified by the kernel before loading
- Cannot crash the system
- Only reads, never modifies system state
- Is open source and auditable

## Supported Distributions

| Distribution | eBPF Support | Notes |
|--------------|--------------|-------|
| Ubuntu 20.04+ | Full | BTF included |
| Debian 11+ | Full | BTF included |
| Fedora 32+ | Full | BTF included |
| RHEL/CentOS 8+ | Full | May need `kernel-debuginfo` |
| Arch Linux | Full | BTF included |
| Alpine | Limited | Needs custom kernel |

## See Also

- [How It Works](/guide/how-it-works) - Architecture overview
- [Process Monitoring](/features/process-monitoring) - Process tracking
- [Troubleshooting](/reference/troubleshooting) - Common issues
