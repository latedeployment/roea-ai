# Prevention Mode Architecture Plan

## Overview

Prevention mode is an advanced feature that allows roea-agent to **block** (not just alert on) access to protected files by AI coding agents. This document outlines the architecture and implementation plan.

## Current State: Detection Mode

Currently, roea-agent operates in **detection mode**:
- Monitors file access via `/proc/*/fd` polling
- Generates alerts when protected files are accessed
- No intervention - access is allowed to proceed

```
[AI Agent] --access--> [Protected File]
                            |
                       [Detected]
                            |
                       [Alert Generated]
```

## Target State: Prevention Mode

Prevention mode will **block** access attempts:

```
[AI Agent] --access--> [roea-agent intercept] --DENY--> [Access Blocked]
                            |
                       [Alert Generated]
```

## Implementation Approaches

### Approach 1: eBPF-based LSM (Linux Security Module)

**Recommended for Linux**

Use eBPF LSM hooks to intercept and block file operations in the kernel.

**Architecture:**
```
┌─────────────────────────────────────────────────────────┐
│                     Kernel Space                        │
│  ┌─────────────────────────────────────────────────┐   │
│  │              eBPF LSM Program                    │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────────────┐  │   │
│  │  │file_open│  │file_read│  │file_permission  │  │   │
│  │  └────┬────┘  └────┬────┘  └────────┬────────┘  │   │
│  │       │            │                 │           │   │
│  │       └────────────┼─────────────────┘           │   │
│  │                    ▼                              │   │
│  │           ┌───────────────┐                      │   │
│  │           │ Protection Map │◄──── Protected PIDs │   │
│  │           │   (eBPF Map)   │◄──── Protected Paths│   │
│  │           └───────┬───────┘                      │   │
│  │                   │                               │   │
│  │           ┌───────▼───────┐                      │   │
│  │           │ Decision Logic │                      │   │
│  │           │ Return -EACCES │                      │   │
│  │           │ or Allow       │                      │   │
│  │           └───────────────┘                      │   │
│  └─────────────────────────────────────────────────┘   │
│                         │                               │
│                    Ring Buffer                          │
│                         │                               │
└─────────────────────────┼───────────────────────────────┘
                          ▼
┌─────────────────────────────────────────────────────────┐
│                     User Space                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │                 roea-agent                       │   │
│  │  ┌──────────────┐  ┌──────────────────────────┐ │   │
│  │  │ eBPF Loader  │  │ Protection Config        │ │   │
│  │  │              │  │ - Protected paths        │ │   │
│  │  │ - Load prog  │  │ - AI agent PIDs          │ │   │
│  │  │ - Update maps│  │ - Policy rules           │ │   │
│  │  └──────────────┘  └──────────────────────────┘ │   │
│  │                                                  │   │
│  │  ┌──────────────────────────────────────────┐   │   │
│  │  │              TUI / Logging                │   │   │
│  │  │  - Display blocked attempts               │   │   │
│  │  │  - Real-time alerts                       │   │   │
│  │  └──────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

**Required LSM Hooks:**
- `file_open` - Block opening protected files
- `file_permission` - Block read/write operations
- `inode_permission` - Low-level permission checks
- `path_truncate` - Block file truncation
- `path_unlink` - Block file deletion

**eBPF Maps:**
```c
// Map of protected path prefixes (hashed)
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 1024);
    __type(key, u64);    // path hash
    __type(value, u8);   // protection level
} protected_paths SEC(".maps");

// Map of AI agent PIDs to monitor
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 256);
    __type(key, u32);    // pid
    __type(value, u8);   // is_agent
} tracked_pids SEC(".maps");

// Ring buffer for events to userspace
struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 256 * 1024);
} events SEC(".maps");
```

**Requirements:**
- Linux kernel 5.7+ with BTF support
- CONFIG_BPF_LSM=y
- CAP_BPF and CAP_MAC_ADMIN capabilities
- Kernel command line: `lsm=bpf,...`

### Approach 2: FUSE (Filesystem in Userspace)

**Alternative approach for any platform**

Create a FUSE filesystem overlay that intercepts file operations.

**Architecture:**
```
┌────────────────────────────────────────────────┐
│                  Application                    │
│                       │                         │
│                   open("/protected/file")       │
│                       ▼                         │
└───────────────────────┬─────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────┐
│                 VFS Layer                        │
│                       │                         │
└───────────────────────┬─────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────┐
│              FUSE Kernel Module                  │
│                       │                         │
└───────────────────────┬─────────────────────────┘
                        │
          ┌─────────────┴─────────────┐
          ▼                           ▼
┌─────────────────────┐     ┌─────────────────────┐
│   roea-fuse daemon  │     │   Underlying FS     │
│                     │     │                     │
│  - Check PID        │     │  /actual/path       │
│  - Check policy     │     │                     │
│  - Block or allow   │     │                     │
└─────────────────────┘     └─────────────────────┘
```

**Pros:**
- Works on Linux, macOS, Windows (with WinFsp)
- No kernel modifications needed
- Easier to implement

**Cons:**
- Performance overhead
- Must mount overlay on protected directories
- Can be bypassed via direct device access

### Approach 3: ptrace-based Interception

**Debug-based approach**

Use ptrace to intercept system calls from AI agent processes.

**Architecture:**
```
┌─────────────────────────────────────────────────┐
│                 roea-agent                       │
│  ┌─────────────────────────────────────────┐    │
│  │           ptrace Interceptor            │    │
│  │                                         │    │
│  │  1. Attach to AI agent process          │    │
│  │  2. Intercept open(), openat(), etc.    │    │
│  │  3. Check if path is protected          │    │
│  │  4. Modify return value to -EACCES      │    │
│  │     or allow syscall to proceed         │    │
│  └─────────────────────────────────────────┘    │
│                       │                          │
│                       ▼                          │
│  ┌─────────────────────────────────────────┐    │
│  │         AI Agent Process (tracee)       │    │
│  │                                         │    │
│  │  All syscalls intercepted by tracer     │    │
│  └─────────────────────────────────────────┘    │
└─────────────────────────────────────────────────┘
```

**Pros:**
- Works on any Linux system
- No special kernel features needed
- Can be very precise

**Cons:**
- Significant performance overhead
- Complex implementation (handle all syscall variants)
- One tracer per AI agent process
- Process must be attached before operations

## Recommended Implementation Path

### Phase 1: Detection Enhancement (Current)
- Improve file monitoring accuracy
- Add real-time alerting in TUI
- Log all protected file access attempts

### Phase 2: eBPF LSM Prevention (Linux)
```rust
// Add to roea-agent/src/prevention/mod.rs

pub struct PreventionEngine {
    /// eBPF LSM program handle
    lsm_program: Option<LsmProgram>,
    /// Protected paths map
    protected_paths: ProtectedPathsMap,
    /// Tracked AI agent PIDs
    tracked_pids: TrackedPidsMap,
    /// Event ring buffer
    events: RingBuffer<PreventionEvent>,
}

impl PreventionEngine {
    pub fn new() -> Result<Self> { ... }

    /// Load and attach eBPF LSM program
    pub fn enable(&mut self) -> Result<()> { ... }

    /// Update protected paths from config
    pub fn update_protected_paths(&mut self, paths: &[PathBuf]) -> Result<()> { ... }

    /// Update tracked AI agent PIDs
    pub fn update_tracked_pids(&mut self, pids: &[u32]) -> Result<()> { ... }

    /// Poll for prevention events
    pub fn poll_events(&mut self) -> Vec<PreventionEvent> { ... }
}

pub struct PreventionEvent {
    pub timestamp: DateTime<Utc>,
    pub pid: u32,
    pub path: String,
    pub operation: FileOperation,
    pub blocked: bool,
    pub agent_name: Option<String>,
}
```

### Phase 3: Cross-Platform Prevention
- FUSE-based prevention for macOS/Windows
- Unified prevention API across platforms

## Configuration

Add to protection config:

```toml
# Enable prevention mode (block instead of just alert)
prevention_mode = true

# Prevention policy
[prevention]
# What to do when access is detected
action = "block"  # "block", "alert", "log"

# Allow list - never block these processes
allow_processes = [
    "code",      # VS Code (user editing)
    "vim",
    "nano",
]

# Exceptions - allow certain AI agents to access certain paths
[[prevention.exceptions]]
agent = "claude_code"
paths = ["/home/user/.config/claude/*"]

# Grace period - allow access for N seconds after user confirmation
grace_period_seconds = 30
```

## User Experience

### TUI Prevention Display

```
┌─────────────────────────────────────────────────────────┐
│ roea-ai │ AI Agent Monitor ━━━ Prevention Mode ACTIVE   │
╞═════════════════════════════════════════════════════════╡
│ ▶ Agents: 2  │  Blocked: 5  │  Allowed: 142  │  🔒 ON   │
├─────────────────────────────────────────────────────────┤
│ [1] Events  [2] Processes  [3] Network  [4] 🚫 Blocked  │
├─────────────────────────────────────────────────────────┤
│ Time      Type    PID   Process       Path              │
│ ─────────────────────────────────────────────────────── │
│ 14:32:01  🚫 BLOCK  1234  claude_code   /etc/passwd     │
│ 14:32:00  🚫 BLOCK  1234  claude_code   /etc/shadow     │
│ 14:31:45  ✓ ALLOW  1234  claude_code   /home/user/code │
│ 14:31:30  🚫 BLOCK  5678  cursor        ~/.ssh/id_rsa   │
└─────────────────────────────────────────────────────────┘
```

### Interactive Prompts (Future)

```
┌───────────────────────────────────────────────────────────┐
│                    ⚠️  ACCESS REQUEST                     │
├───────────────────────────────────────────────────────────┤
│                                                           │
│  Claude Code (PID 1234) is trying to access:              │
│                                                           │
│    /home/user/.ssh/id_rsa                                 │
│                                                           │
│  This file is marked as protected.                        │
│                                                           │
│  ┌─────────┐  ┌─────────┐  ┌─────────────────────┐       │
│  │  Allow  │  │  Block  │  │  Allow for session  │       │
│  └─────────┘  └─────────┘  └─────────────────────┘       │
│                                                           │
│  Press [A] Allow  [B] Block  [S] Allow for session       │
└───────────────────────────────────────────────────────────┘
```

## Security Considerations

1. **Privilege Requirements**: Prevention mode requires elevated privileges
   - Linux: CAP_BPF, CAP_MAC_ADMIN, CAP_SYS_ADMIN
   - Run as root or with specific capabilities

2. **Bypass Risks**:
   - Direct device access (`/dev/sda`)
   - Memory mapping existing file descriptors
   - Symbolic link traversal
   - Race conditions (TOCTOU)

3. **Mitigations**:
   - Block at inode level, not just path
   - Monitor /dev access
   - Track file descriptor inheritance
   - Use seccomp for additional hardening

4. **Audit Trail**:
   - Log all prevention decisions
   - Include process ancestry
   - Store in tamper-resistant format

## Implementation Timeline

| Phase | Feature | Status |
|-------|---------|--------|
| 1.0 | Detection mode with TUI | ✅ Implemented |
| 1.1 | Enhanced file monitoring | ✅ Implemented |
| 1.2 | Protection config (TOML) | ✅ Implemented |
| 2.0 | eBPF LSM infrastructure | 📋 Planned |
| 2.1 | Basic file_open blocking | 📋 Planned |
| 2.2 | Full file operation blocking | 📋 Planned |
| 2.3 | Interactive prompts | 📋 Planned |
| 3.0 | FUSE cross-platform | 📋 Planned |

## Testing Strategy

1. **Unit Tests**: Mock eBPF interactions
2. **Integration Tests**: Use namespaces/containers
3. **Security Tests**: Bypass attempt verification
4. **Performance Tests**: Measure syscall overhead

## References

- [eBPF LSM Documentation](https://docs.kernel.org/bpf/prog_lsm.html)
- [Linux Security Modules](https://www.kernel.org/doc/html/latest/admin-guide/LSM/index.html)
- [FUSE Documentation](https://libfuse.github.io/doxygen/)
- [Cilium BPF and XDP Reference](https://docs.cilium.io/en/stable/bpf/)
