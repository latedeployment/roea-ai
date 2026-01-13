---
layout: home

hero:
  name: roea-ai
  text: Observability for AI Coding Agents
  tagline: See what Claude Code, Cursor, Copilot, and other AI agents are really doing on your system
  image:
    src: /hero-image.svg
    alt: roea-ai
  actions:
    - theme: brand
      text: Get Started
      link: /guide/introduction
    - theme: alt
      text: View on GitHub
      link: https://github.com/your-org/roea-ai

features:
  - icon: "🔍"
    title: Process Tree Visualization
    details: Real-time graph showing all processes spawned by AI agents, their parent-child relationships, and lifecycle events
  - icon: "🌐"
    title: Network Connection Tracking
    details: Monitor all network connections - see which APIs your agents call, identify unexpected outbound traffic
  - icon: "📁"
    title: File Access Monitoring
    details: Track every file read, write, and modification made by AI coding agents across your codebase
  - icon: "🤖"
    title: Multi-Agent Support
    details: Built-in detection for Claude Code, Cursor, VS Code Copilot, Windsurf, Aider, and more
  - icon: "⚡"
    title: High Performance
    details: eBPF-powered monitoring on Linux for minimal overhead, cross-platform support for macOS and Windows
  - icon: "🔒"
    title: Privacy First
    details: All data stays local on your machine, no cloud dependency, full control over your monitoring data
---

## Why roea-ai?

AI coding agents are increasingly powerful, but they operate as black boxes. They spawn processes, make network calls, and access files - often in ways that aren't visible to users.

**roea-ai** provides the visibility you need:

- **Security**: Know exactly what processes and network calls your AI agents make
- **Debugging**: Understand agent behavior when things go wrong
- **Compliance**: Audit AI agent activity for regulated environments
- **Learning**: See how agents work under the hood

## Quick Example

```bash
# Install roea-ai
brew install roea-ai  # macOS
# or
curl -sSL https://roea.ai/install.sh | bash  # Linux

# Start the daemon
roea-agent

# Open the UI
roea-ui
```

Then use any AI coding agent - Claude Code, Cursor, Copilot - and watch roea-ai visualize everything in real-time.

## Supported Agents

<div class="agents-grid">

| Agent | Status | Detection Method |
|-------|--------|------------------|
| Claude Code | Full Support | Process + Network |
| Cursor IDE | Full Support | Process Tree |
| VS Code Copilot | Full Support | Extension Detection |
| Windsurf | Full Support | Process Name |
| Aider | Full Support | CLI Detection |
| Continue.dev | Full Support | Extension Detection |
| Cline | Full Support | Extension Detection |

</div>

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Desktop UI (Tauri)                   │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────────────┐ │
│  │  Sidebar    │  │ Process     │  │   Details Panel      │ │
│  │  (Agents)   │  │ Graph (D3)  │  │   (Connections/Files)│ │
│  └─────────────┘  └─────────────┘  └──────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │ gRPC
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    roea-agent (Rust Daemon)                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────┐  │
│  │ Process  │  │ Network  │  │   File   │  │  Signature  │  │
│  │ Monitor  │  │ Monitor  │  │ Monitor  │  │  Matcher    │  │
│  └──────────┘  └──────────┘  └──────────┘  └─────────────┘  │
│                              │                               │
│                       ┌──────┴──────┐                       │
│                       │   DuckDB    │                       │
│                       │   Storage   │                       │
│                       └─────────────┘                       │
└─────────────────────────────────────────────────────────────┘
```
