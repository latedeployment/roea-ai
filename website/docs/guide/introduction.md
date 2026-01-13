# Introduction

## What is roea-ai?

**roea-ai** (pronounced "ROY-ah") is an open-source observability tool designed specifically for AI coding agents. It provides real-time visibility into what AI agents like Claude Code, Cursor, VS Code Copilot, and others are doing on your system.

## The Problem

AI coding agents are powerful productivity tools, but they operate as black boxes:

- **No visibility**: You can't see what processes they spawn
- **Hidden network calls**: API calls to AI services, package registries, and unknown endpoints
- **File access opacity**: Which files are being read and written during a session?
- **Security concerns**: How do you audit AI agent behavior?

Traditional system monitoring tools aren't designed for this use case - they show everything, making it hard to focus on AI agent activity specifically.

## The Solution

roea-ai solves this by:

1. **Agent-Aware Monitoring**: Automatically detects and tracks AI coding agents
2. **Process Tree Visualization**: Shows the full hierarchy of processes spawned by agents
3. **Network Connection Tracking**: Monitors all network activity with endpoint classification
4. **File Access Logging**: Records all file operations with filtering for noise
5. **Local-First Architecture**: All data stays on your machine

## Key Features

### Real-Time Process Graph

<div class="feature-preview">

![Process Graph](/screenshots/process-graph.png)

</div>

Interactive D3.js visualization showing:
- Process hierarchy (parent → child relationships)
- Active vs. exited processes
- AI agent identification with color coding
- Expandable details for any process

### Network Connection Tracking

Monitor all network connections with automatic classification:

| Endpoint Type | Examples |
|---------------|----------|
| LLM APIs | api.anthropic.com, api.openai.com |
| GitHub | api.github.com, github.com |
| Package Registries | registry.npmjs.org, pypi.org, crates.io |
| Telemetry | sentry.io, segment.com |

### File Access Monitoring

Track every file operation:
- **Reads**: Files accessed by the agent
- **Writes**: Files created or modified
- **Noise filtering**: Automatically filters temp files, caches, etc.

## Architecture Overview

roea-ai consists of two main components:

```
┌──────────────────────┐
│   Desktop UI (Tauri) │  ◄── React + D3.js visualization
└──────────┬───────────┘
           │ gRPC
┌──────────▼───────────┐
│   roea-agent (Rust)  │  ◄── Background daemon
│  ┌─────────────────┐ │
│  │ Process Monitor │ │  ◄── eBPF on Linux, sysinfo elsewhere
│  │ Network Monitor │ │  ◄── TCP/UDP/Unix socket tracking
│  │ File Monitor    │ │  ◄── File operation logging
│  │ DuckDB Storage  │ │  ◄── Local embedded database
│  └─────────────────┘ │
└──────────────────────┘
```

## Supported Platforms

| Platform | Process Monitor | Network Monitor | File Monitor |
|----------|----------------|-----------------|--------------|
| Linux | eBPF (kernel 5.8+) or sysinfo | /proc/net parsing | /proc/fd parsing |
| macOS | sysinfo | lsof-based | lsof-based |
| Windows | sysinfo | netstat-based | (coming soon) |

## Privacy & Security

roea-ai is designed with privacy in mind:

- **100% local**: No data ever leaves your machine
- **No cloud dependency**: Works completely offline
- **Open source**: Full code transparency
- **Sensitive data filtering**: API keys and tokens are redacted from logs

## Next Steps

Ready to get started?

1. [Installation Guide](/guide/installation) - Set up roea-ai on your system
2. [Quick Start](/guide/quick-start) - Run your first monitoring session
3. [System Requirements](/guide/requirements) - Check platform compatibility
