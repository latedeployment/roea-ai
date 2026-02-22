# tuai

**[k9s](https://k9scli.io/) for AI agents** - see what your AI coding tools are doing on your system.

tuai is a terminal UI that monitors AI coding agents in real-time. It tracks processes, network connections, and file access - like [k9s](https://k9scli.io/) but for AI agents instead of Kubernetes pods.

```
 tuai  2 agents | 847 procs | 14 net | 203 files | 0 alerts            up 00:12:34
 [1] Agents | [2] Events | [3] Network | [4] Alerts
╭ Agents (2) ──────────────────────────────────────────────────────────────────────╮
│ PID      NAME                 TYPE           CWD                     CMDLINE     │
│ 14523    claude_code          Claude Code    /home/user/project      claude ...  │
│ 15891    cursor               Cursor         /home/user/app          cursor ...  │
╰──────────────────────────────────────────────────────────────────────────────────╯
 <?> help  </> search  <j/k> scroll  <f> filter  <c> clear  <q> quit
```

## Features

- **Agent detection** - auto-detects Claude Code, Cursor, Aider, Windsurf, Copilot, Continue.dev, Ollama, LM Studio, LocalAI
- **Process tracking** - monitors agent processes and their child processes
- **Network monitoring** - tracks connections made by agents
- **File access logging** - see which files agents read/write
- **Protected file alerts** - get alerted when agents access sensitive files (`/etc/shadow`, SSH keys, `.env`, etc.)
- **eBPF support** - optional kernel-level monitoring on Linux for zero-overhead tracing
- **Cross-platform** - works on Linux and macOS (sysinfo-based fallback)

## Install

```bash
cargo install tuai
```

Or build from source:

```bash
git clone https://github.com/latedeployment/tuai
cd tuai
cargo build --release -p tuai
```

Then run it:

```bash
cargo run --release -p tuai
```

## Usage

```bash
# Launch TUI (default)
tuai

# With file protection monitoring
tuai --protect-config protect.toml

# Stream events as JSON (for piping)
tuai --events

# Run gRPC server mode
tuai --server
```

### Keybindings

| Key | Action |
|-----|--------|
| `1-4` | Switch view (Agents, Events, Network, Alerts) |
| `j/k` | Scroll down/up |
| `/` | Search |
| `f` | Cycle severity filter |
| `g/G` | Jump to top/bottom |
| `c` | Clear events |
| `q` | Quit |

## Views

- **Agents** - tracked AI agent processes with PID, type, working directory
- **Events** - live event log with severity, timestamps, and details
- **Network** - all network connections made by tracked agents
- **Alerts** - protected file access alerts

## File Protection

Configure sensitive files to monitor:

```toml
# protect.toml
include_defaults = true  # /etc/passwd, SSH keys, etc.
alert_severity = "critical"

files = [
    "~/.aws/credentials",
    "~/.config/gh/hosts.yml",
]

patterns = [
    "**/.env",
    "**/*.pem",
    "**/*.key",
]
```

## eBPF (Linux)

On Linux with kernel 5.8+, tuai can use eBPF for kernel-level monitoring:

```bash
# Generate vmlinux.h (one-time)
bpftool btf dump file /sys/kernel/btf/vmlinux format c > crates/tuai/src/bpf/vmlinux.h

# Run with eBPF
sudo tuai
```

## License

MIT
