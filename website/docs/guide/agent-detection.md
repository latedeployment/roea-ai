# Agent Detection

roea-ai automatically detects AI coding agents running on your system using a configurable signature-based system. This page explains how detection works and how to customize it.

## How Detection Works

When a new process starts, roea-ai checks it against a database of agent signatures. Each signature defines patterns that identify a specific AI agent.

```
New Process Event
       │
       ▼
┌──────────────────┐
│ Check Process    │──── Match ───▶ Tag as Agent
│ Name Patterns    │
└──────────────────┘
       │ No Match
       ▼
┌──────────────────┐
│ Check Command    │──── Match ───▶ Tag as Agent
│ Line Patterns    │
└──────────────────┘
       │ No Match
       ▼
┌──────────────────┐
│ Check Executable │──── Match ───▶ Tag as Agent
│ Path Patterns    │
└──────────────────┘
       │ No Match
       ▼
    Not an Agent
```

## Built-in Agent Signatures

roea-ai includes signatures for popular AI coding agents:

| Agent | Detection Method | Child Tracking |
|-------|-----------------|----------------|
| **Claude Code** | Process name `claude`, cmdline patterns | Yes |
| **Cursor IDE** | Executable path, helper processes | Yes |
| **VS Code + Copilot** | Copilot extension process | Yes |
| **Windsurf** | Executable path | Yes |
| **Aider** | Process name `aider`, Python patterns | Yes |
| **Continue.dev** | VS Code extension process | Yes |
| **Cline** | VS Code extension process | Yes |

### Claude Code

```yaml
name: claude-code
display_name: Claude Code
icon: /icons/claude.svg
process_patterns:
  - "^claude$"
  - "claude-code"
command_patterns:
  - "claude\\s+(chat|code|api)"
  - "--api-key"
network_endpoints:
  - "api.anthropic.com"
track_children: true
parent_hints:
  - "zsh"
  - "bash"
  - "fish"
  - "terminal"
```

### Cursor IDE

```yaml
name: cursor
display_name: Cursor
icon: /icons/cursor.svg
process_patterns:
  - "^[Cc]ursor$"
  - "Cursor Helper"
  - "cursor-helper"
exe_path_patterns:
  - "Cursor\\.app"
  - "cursor/cursor"
  - "AppData.*[Cc]ursor"
network_endpoints:
  - "api.cursor.sh"
  - "api2.cursor.sh"
track_children: true
```

### VS Code + Copilot

```yaml
name: copilot
display_name: GitHub Copilot
icon: /icons/copilot.svg
process_patterns:
  - "copilot"
  - "github\\.copilot"
exe_path_patterns:
  - "extensions.*copilot"
network_endpoints:
  - "copilot-proxy.githubusercontent.com"
  - "api.github.com"
track_children: false
parent_hints:
  - "code"
  - "Code Helper"
```

## Signature File Format

Agent signatures are defined in YAML files. Here's the complete schema:

```yaml
# Required fields
name: string              # Unique identifier
display_name: string      # Human-readable name

# Optional identification patterns (at least one required)
process_patterns:         # Regex patterns for process name
  - "pattern1"
  - "pattern2"

command_patterns:         # Regex patterns for command line
  - "pattern1"

exe_path_patterns:        # Regex patterns for executable path
  - "pattern1"

# Optional metadata
icon: string              # Path to icon file
color: string             # Hex color code for UI

# Network tracking
network_endpoints:        # Known API endpoints
  - "api.example.com"

# Process tree tracking
track_children: boolean   # Track child processes (default: false)
parent_hints:             # Expected parent process names
  - "shell"
  - "terminal"
```

## Pattern Matching Rules

### Process Name Patterns

Process name patterns match against the executable name (e.g., `claude`, `cursor`):

```yaml
process_patterns:
  - "^claude$"        # Exact match
  - "cursor"          # Contains "cursor"
  - "^aider"          # Starts with "aider"
```

### Command Line Patterns

Command line patterns match against the full command line including arguments:

```yaml
command_patterns:
  - "claude\\s+chat"      # "claude chat" or "claude  chat"
  - "--model\\s+opus"     # Model flag
  - "-k\\s+sk-"           # API key flag
```

### Executable Path Patterns

Path patterns match against the full executable path:

```yaml
exe_path_patterns:
  - "/Applications/Cursor\\.app"
  - "AppData.*[Cc]ursor"
  - "/usr/local/bin/aider"
```

## Child Process Tracking

When `track_children: true`, roea-ai tracks all processes spawned by the agent:

```
Claude Code (PID: 1234)
├── bash (PID: 1235)    ← Tracked as Claude child
│   └── npm (PID: 1236) ← Tracked as Claude descendant
├── node (PID: 1237)    ← Tracked as Claude child
└── git (PID: 1238)     ← Tracked as Claude child
```

This is essential for understanding what tools an AI agent invokes.

## Adding Custom Signatures

### Method 1: Configuration File

Add signatures to your `roea.toml` config:

```toml
[[agent_signatures]]
name = "my-custom-agent"
display_name = "My Custom Agent"
process_patterns = ["my-agent", "custom-ai"]
command_patterns = ["--custom-flag"]
track_children = true
```

### Method 2: Signature Directory

Create a YAML file in `~/.config/roea/signatures/`:

```yaml
# ~/.config/roea/signatures/my-agent.yaml
name: my-agent
display_name: My Custom AI Agent
process_patterns:
  - "my-agent"
  - "custom-ai-tool"
network_endpoints:
  - "api.my-ai.com"
track_children: true
```

### Method 3: Environment Variable

Set the `ROEA_SIGNATURES_PATH` environment variable:

```bash
export ROEA_SIGNATURES_PATH=/path/to/signatures:/another/path
```

## Troubleshooting Detection

### Agent Not Detected

1. **Check process name**: Run `ps aux | grep <agent>` to see exact process name
2. **Check command line**: Look at `/proc/<pid>/cmdline` on Linux
3. **Add debug logging**: Set `RUST_LOG=roea_agent::signatures=debug`

### False Positives

If non-agent processes are being detected:

1. Make patterns more specific with anchors (`^`, `$`)
2. Add negative patterns using `!` prefix
3. Use `exe_path_patterns` for more precise matching

### Detection Latency

If detection seems slow:

1. Ensure eBPF is enabled on Linux (check logs)
2. Reduce polling interval in config
3. Check system load

## API Access

Query detected agents programmatically:

```bash
# gRPC query
grpcurl -plaintext localhost:50051 roea.RoeaAgent/GetAgentSignatures

# Via roea-cli
roea-cli agents list
roea-cli agents show claude-code
```

## Next Steps

- [Process Monitoring](/features/process-monitoring) - Detailed process tracking
- [Configuration](/reference/configuration) - Full configuration reference
- [Troubleshooting](/reference/troubleshooting) - Common issues
