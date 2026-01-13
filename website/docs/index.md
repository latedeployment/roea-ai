---
layout: home

hero:
  name: roea-ai
  text: See What Your AI Agents Are Really Doing
  tagline: Real-time observability for Claude Code, Cursor, Copilot, and other AI coding agents. Process trees, network calls, file access - all in one view.
  image:
    src: /hero-image.svg
    alt: roea-ai process visualization
  actions:
    - theme: brand
      text: Download for Free
      link: /download
    - theme: alt
      text: View on GitHub
      link: https://github.com/your-org/roea-ai
    - theme: alt
      text: Read the Docs
      link: /guide/introduction

features:
  - icon: "🔍"
    title: Process Tree Visualization
    details: Watch AI agents spawn processes in real-time. See the full hierarchy from parent to child, track lifecycle events, and understand exactly what's running.
    link: /features/process-monitoring
    linkText: Learn more
  - icon: "🌐"
    title: Network Connection Tracking
    details: Monitor every API call - Anthropic, OpenAI, GitHub, npm registries. Know where your data goes and identify unexpected network traffic.
    link: /features/network-tracking
    linkText: Learn more
  - icon: "📁"
    title: File Access Monitoring
    details: Track file reads, writes, and modifications. See which files AI agents touch in your codebase. Filter noise automatically.
    link: /features/file-access
    linkText: Learn more
  - icon: "🤖"
    title: Multi-Agent Support
    details: Built-in detection for Claude Code, Cursor, VS Code Copilot, Windsurf, Aider, Continue.dev, and Cline. Add custom agents easily.
    link: /features/agents/claude-code
    linkText: See all agents
  - icon: "⚡"
    title: High Performance
    details: eBPF-powered monitoring on Linux for sub-millisecond detection with minimal overhead. Native support for macOS and Windows.
    link: /guide/requirements
    linkText: System requirements
  - icon: "🔒"
    title: 100% Local & Private
    details: All data stays on your machine. No cloud, no tracking, no dependencies. Open source and fully auditable.
    link: https://github.com/your-org/roea-ai
    linkText: View source
---

<div class="vp-doc">

## Download

Get roea-ai for your platform:

<div class="download-buttons">
  <a href="https://github.com/your-org/roea-ai/releases/latest/download/roea-ai_aarch64.dmg" class="download-btn">
    <svg viewBox="0 0 24 24" fill="currentColor"><path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z"/></svg>
    macOS (Apple Silicon)
  </a>
  <a href="https://github.com/your-org/roea-ai/releases/latest/download/roea-ai_x64.dmg" class="download-btn">
    <svg viewBox="0 0 24 24" fill="currentColor"><path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z"/></svg>
    macOS (Intel)
  </a>
  <a href="https://github.com/your-org/roea-ai/releases/latest/download/roea-ai_x64_en-US.msi" class="download-btn">
    <svg viewBox="0 0 24 24" fill="currentColor"><path d="M3 12V6.75l6-1.32v6.48L3 12zm17-9v8.75l-10 .15V5.21L20 3zM3 13l6 .09v6.81l-6-1.15V13zm7 .17l10 .05v7.78l-10-1.52V13.17z"/></svg>
    Windows
  </a>
  <a href="https://github.com/your-org/roea-ai/releases/latest/download/roea-ai_amd64.deb" class="download-btn">
    <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/></svg>
    Linux (.deb)
  </a>
</div>

<p style="text-align: center; color: var(--vp-c-text-2); font-size: 0.875rem;">
  <a href="https://github.com/your-org/roea-ai/releases" style="color: inherit;">View all releases</a> · MIT License · Open Source
</p>

---

## Why roea-ai?

AI coding agents are **black boxes**. They spawn processes, make API calls, and access files - but you can't see what they're doing.

<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 24px; margin: 32px 0;">

<div style="padding: 24px; background: var(--vp-c-bg-soft); border-radius: 12px; border: 1px solid var(--vp-c-divider);">
<h3 style="margin-top: 0;">🔐 Security</h3>
<p style="color: var(--vp-c-text-2); margin-bottom: 0;">Know exactly what processes your AI agents spawn and what network calls they make. Identify unexpected behavior before it becomes a problem.</p>
</div>

<div style="padding: 24px; background: var(--vp-c-bg-soft); border-radius: 12px; border: 1px solid var(--vp-c-divider);">
<h3 style="margin-top: 0;">🐛 Debugging</h3>
<p style="color: var(--vp-c-text-2); margin-bottom: 0;">When AI agents behave unexpectedly, see the full process tree, network requests, and file access to understand what went wrong.</p>
</div>

<div style="padding: 24px; background: var(--vp-c-bg-soft); border-radius: 12px; border: 1px solid var(--vp-c-divider);">
<h3 style="margin-top: 0;">📋 Compliance</h3>
<p style="color: var(--vp-c-text-2); margin-bottom: 0;">Audit AI agent activity for regulated environments. Export detailed logs of all process and network activity.</p>
</div>

<div style="padding: 24px; background: var(--vp-c-bg-soft); border-radius: 12px; border: 1px solid var(--vp-c-divider);">
<h3 style="margin-top: 0;">🎓 Learning</h3>
<p style="color: var(--vp-c-text-2); margin-bottom: 0;">Understand how AI coding agents work under the hood. See the tools they use, the APIs they call, and how they interact with your system.</p>
</div>

</div>

---

## Supported Agents

roea-ai automatically detects and tracks these AI coding agents:

| Agent | Detection | Process Tree | Network | Files |
|-------|-----------|--------------|---------|-------|
| **Claude Code** | Full | ✅ | ✅ | ✅ |
| **Cursor IDE** | Full | ✅ | ✅ | ✅ |
| **VS Code Copilot** | Full | ✅ | ✅ | ✅ |
| **Windsurf** | Full | ✅ | ✅ | ✅ |
| **Aider** | Full | ✅ | ✅ | ✅ |
| **Continue.dev** | Full | ✅ | ✅ | ✅ |
| **Cline** | Full | ✅ | ✅ | ✅ |

Don't see your agent? [Add a custom signature](/reference/configuration) or [open an issue](https://github.com/your-org/roea-ai/issues).

---

## Quick Start

Get up and running in under a minute:

```bash
# 1. Download and install (see above)

# 2. Start the monitoring daemon
roea-agent

# 3. Open the desktop UI
roea-ui

# 4. Use any AI coding agent
claude  # or cursor, aider, etc.

# 5. Watch the magic happen! 🎉
```

See the full [Quick Start Guide](/guide/quick-start) for more details.

---

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

**Built with:**
- 🦀 **Rust** - High-performance daemon with eBPF support
- ⚛️ **React** - Modern UI with D3.js visualizations
- 🖥️ **Tauri** - Native cross-platform desktop app
- 🦆 **DuckDB** - Fast embedded analytics database

---

<div class="cta-section">
<h2>Ready to see what your AI agents are doing?</h2>
<p>Download roea-ai for free and get complete visibility into your AI coding workflows.</p>
<div class="download-buttons" style="justify-content: center;">
  <a href="/download" class="download-btn" style="background: var(--vp-c-brand-1); color: white; border-color: var(--vp-c-brand-1);">
    Download Now
  </a>
  <a href="https://github.com/your-org/roea-ai" class="download-btn">
    <svg viewBox="0 0 24 24" fill="currentColor" style="width: 20px; height: 20px;"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
    Star on GitHub
  </a>
</div>
</div>

</div>
