# FAQ

Frequently asked questions about roea-ai.

## General

### What is roea-ai?

roea-ai is an observability tool for AI coding agents. It monitors processes, network connections, and file access from tools like Claude Code, Cursor, Copilot, and Aider, giving you visibility into what these agents are doing on your system.

### Is roea-ai free?

Yes! roea-ai is free and open source under the MIT license. There are no paid tiers or feature restrictions.

### Does roea-ai send my data anywhere?

No. All data stays 100% on your machine. roea-ai has no cloud component, no telemetry, and no network communication except to your local gRPC server. You can verify this by reviewing the source code.

### What platforms are supported?

- **macOS**: 11.0 (Big Sur) and later
- **Windows**: Windows 10 (1903) and later
- **Linux**: Kernel 4.18+ (5.8+ recommended for eBPF)

### What AI agents are supported?

Built-in support for:
- Claude Code
- Cursor IDE
- VS Code + GitHub Copilot
- Windsurf
- Aider
- Continue.dev
- Cline

You can also add custom agent signatures.

## Security

### Does roea-ai have access to my code?

roea-ai can see file paths that are accessed by monitored processes. It does not read file contents. File monitoring tracks which files are opened/written, not what's inside them.

### Does roea-ai see my API keys?

roea-ai redacts sensitive information (API keys, tokens, passwords) from displayed command lines and logs. The raw data may contain this information in the database, but it's filtered in the UI and exports.

### Can roea-ai modify my system?

No. roea-ai is read-only. It only observes system activity and never modifies files, processes, or network connections.

### Is it safe to run with elevated privileges?

roea-ai's eBPF programs are verified by the Linux kernel and cannot crash or compromise your system. However, as with any software requesting elevated privileges, review the source code if you're concerned.

## Technical

### Why do I need eBPF on Linux?

eBPF provides:
- Sub-millisecond process detection
- No missed events
- Lower CPU overhead than polling

Without eBPF, roea-ai falls back to polling-based monitoring which works but has higher latency and may miss short-lived processes.

### How much disk space does roea-ai use?

Default storage grows at roughly 10-50 MB per day depending on activity. You can configure retention to limit storage:

```toml
[storage.retention]
processes = "7d"
connections = "3d"
```

### How much CPU does roea-ai use?

- **Idle**: <0.5% CPU
- **Active monitoring**: 0.5-2% CPU
- **With eBPF**: <1% CPU

### Can I run roea-ai on a server?

Yes. roea-agent can run headless on servers. Use the CLI or gRPC API to query data instead of the desktop UI.

### Does roea-ai work in containers?

Partially. roea-ai can monitor containerized AI agents, but:
- File paths will be container paths
- Some network information may be limited
- eBPF requires host privileges

## Usage

### How do I add a custom AI agent?

Create a signature file:

```yaml
# ~/.config/roea/signatures/my-agent.yaml
name: my-agent
display_name: My Custom Agent
process_patterns:
  - "my-agent-process"
track_children: true
```

See [Agent Detection](/guide/agent-detection) for details.

### Can I export data for analysis?

Yes. Export to JSON, CSV, or Parquet:

```bash
roea-cli export --format json --output data.json
```

### How do I query historical data?

Use the CLI with SQL:

```bash
roea-cli query "SELECT * FROM processes WHERE agent_name = 'claude-code' AND start_time > datetime('now', '-1 day')"
```

### Can I integrate with my existing monitoring stack?

Yes. roea-ai supports:
- OpenTelemetry export (Jaeger, Tempo, etc.)
- osquery integration
- Custom exporters via gRPC API

### How do I update roea-ai?

```bash
# macOS (Homebrew)
brew upgrade roea-ai

# Linux (deb)
curl -LO https://github.com/your-org/roea-ai/releases/latest/download/roea-ai_amd64.deb
sudo dpkg -i roea-ai_amd64.deb

# Windows (manual)
# Download latest from GitHub releases
```

## Troubleshooting

### Why isn't my agent detected?

1. Check the agent is running: `ps aux | grep <agent-name>`
2. Verify signature patterns match: `roea-cli signatures test --process-name <name>`
3. Enable debug logging to see matching attempts

See [Troubleshooting](/reference/troubleshooting) for more.

### Why is the graph empty?

- Check daemon is running: `roea-cli status`
- Verify agents are active: `roea-cli agents list`
- Try refreshing the UI

### How do I report a bug?

Open an issue on [GitHub](https://github.com/your-org/roea-ai/issues) with:
- roea-ai version (`roea-agent --version`)
- Operating system
- Steps to reproduce
- Relevant logs

## Privacy

### What data does roea-ai collect?

roea-ai collects from your local system:
- Process information (name, PID, command line, parent)
- Network connections (local/remote addresses, ports)
- File operations (paths, operation types)

All data stays local.

### Can I disable certain monitoring?

Yes. Disable specific monitors:

```toml
[monitor.file]
enabled = false

[monitor.network]
enabled = false
```

### How do I delete all data?

```bash
# Stop daemon
systemctl --user stop roea-agent

# Delete database
rm ~/.local/share/roea/data.duckdb

# Restart
systemctl --user start roea-agent
```

## Contributing

### How can I contribute?

- Report bugs and feature requests on GitHub
- Submit pull requests
- Improve documentation
- Share your custom agent signatures

See [Contributing](/contributing) for guidelines.

### Is there a roadmap?

Check the [GitHub project board](https://github.com/your-org/roea-ai/projects) for planned features.

## More Questions?

- Check [Troubleshooting](/reference/troubleshooting)
- Search [GitHub Issues](https://github.com/your-org/roea-ai/issues)
- Ask in [Discussions](https://github.com/your-org/roea-ai/discussions)
