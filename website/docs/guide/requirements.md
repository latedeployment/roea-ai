# System Requirements

## Minimum Requirements

| Component | Requirement |
|-----------|-------------|
| **CPU** | 64-bit processor (x86_64 or ARM64) |
| **RAM** | 512 MB available |
| **Disk** | 100 MB for installation |
| **OS** | See platform-specific requirements below |

## Platform Support

### Linux

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| **Kernel** | 4.18+ | 5.8+ (for eBPF) |
| **glibc** | 2.17+ | 2.28+ |
| **Distributions** | Any with glibc | Ubuntu 22.04+, Fedora 38+, Debian 12+ |

#### eBPF Requirements (Optional)

For high-performance kernel-level monitoring:

- Kernel 5.8 or later (for ring buffer support)
- BTF (BPF Type Format) enabled
- Either root privileges or `CAP_BPF` capability

Check if your system supports eBPF:

```bash
# Check kernel version
uname -r

# Check for BTF
ls -la /sys/kernel/btf/vmlinux

# If BTF exists, eBPF is supported
```

Without eBPF, roea-ai falls back to polling-based monitoring using sysinfo.

### macOS

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| **macOS Version** | 11.0 (Big Sur) | 13.0 (Ventura)+ |
| **Architecture** | Intel x86_64 | Apple Silicon (M1/M2/M3) |
| **Xcode CLI** | Not required | Recommended for development |

#### Required Permissions

macOS requires explicit permissions for monitoring:

| Permission | Purpose | How to Grant |
|------------|---------|--------------|
| **Full Disk Access** | File monitoring | System Settings → Privacy → Full Disk Access |
| **Developer Tools** | Process inspection | System Settings → Privacy → Developer Tools |

### Windows

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| **Windows Version** | Windows 10 1903+ | Windows 11 |
| **Architecture** | x86_64 | x86_64 |
| **.NET** | Not required | - |

#### Notes

- Windows Defender may flag initial execution (false positive)
- Administrator rights recommended for full monitoring
- SmartScreen warning on first run is expected

## Supported AI Agents

roea-ai includes built-in signatures for these agents:

| Agent | Version | Detection Method |
|-------|---------|------------------|
| Claude Code | 1.0+ | Process name + cmdline |
| Cursor IDE | 0.40+ | Process tree |
| VS Code Copilot | Any | Extension process detection |
| Windsurf | Any | Process name |
| Aider | 0.30+ | Process name + cmdline |
| Continue.dev | Any | Extension detection |
| Cline | Any | Extension detection |
| OpenHands | Any | Container detection |

Other agents can be added via custom [signature files](/reference/configuration).

## Network Requirements

roea-ai is designed to work offline but benefits from network access for:

| Feature | Network Required | Purpose |
|---------|------------------|---------|
| Basic monitoring | No | Core functionality works offline |
| AI agent detection | No | Local process/file inspection |
| OpenTelemetry export | Yes | Send telemetry to external collectors |
| Update checks | Yes | Check for new versions |

### Firewall Configuration

roea-agent listens on localhost only by default:

| Port | Protocol | Purpose |
|------|----------|---------|
| 50051 | gRPC | UI ↔ Daemon communication |

No external network ports are opened.

## Resource Usage

Typical resource consumption:

| Metric | Idle | Active Monitoring |
|--------|------|-------------------|
| **CPU** | < 1% | 1-5% |
| **RAM** | 50 MB | 100-300 MB |
| **Disk I/O** | Minimal | ~1 MB/min (with DuckDB) |
| **Network** | None | N/A (local only) |

### Performance Modes

| Mode | Resource Usage | Completeness |
|------|----------------|--------------|
| eBPF (Linux) | Very Low | Complete |
| Polling (default) | Low | Complete |
| Minimal | Very Low | Process names only |

## Storage Requirements

roea-ai stores monitoring data locally:

| Component | Location | Size |
|-----------|----------|------|
| Configuration | `~/.config/roea-ai/` | < 1 MB |
| Database | `~/.local/share/roea-ai/roea.db` | 10 MB - 1 GB |
| Logs | `~/.local/share/roea-ai/logs/` | < 10 MB |

### Database Growth

The DuckDB database grows with monitoring activity:

- **Light use** (1-2 agents): ~10 MB/day
- **Heavy use** (many agents, files): ~100 MB/day
- **Automatic cleanup**: Old data purged after 7 days (configurable)

## Compatibility Notes

### Docker/Containers

roea-ai can run inside containers but monitoring is limited:

- Process monitoring works for container processes
- Host process visibility requires privileged mode
- eBPF requires host kernel access

### Virtual Machines

Full support in VMs with these notes:

- eBPF may not work in some VM environments
- Nested virtualization affects performance
- ARM VMs on x86 hosts not supported

### WSL (Windows Subsystem for Linux)

WSL 2 is supported:

- Full Linux functionality
- eBPF works with WSL 2 kernel 5.8+
- Windows UI can connect to WSL daemon

## Next Steps

- [Installation Guide](/guide/installation)
- [Quick Start](/guide/quick-start)
- [Linux eBPF Setup](/reference/ebpf)
