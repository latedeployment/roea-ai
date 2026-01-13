/**
 * Mock data fixtures for E2E testing.
 *
 * These fixtures simulate real AI agent sessions for testing the UI
 * without requiring the actual roea-agent daemon to be running.
 */

import type { Process, Connection, AgentSignature, AgentStatus } from "../../../src/lib/types";

// Agent signatures matching the real signatures
export const mockSignatures: AgentSignature[] = [
  {
    name: "claude-code",
    displayName: "Claude Code",
    icon: "claude-icon",
    expectedEndpoints: ["api.anthropic.com"],
    childProcessTracking: true,
  },
  {
    name: "cursor",
    displayName: "Cursor",
    icon: "cursor-icon",
    expectedEndpoints: ["api.cursor.sh", "api.openai.com"],
    childProcessTracking: true,
  },
  {
    name: "aider",
    displayName: "Aider",
    icon: "aider-icon",
    expectedEndpoints: ["api.openai.com", "api.anthropic.com"],
    childProcessTracking: true,
  },
  {
    name: "windsurf",
    displayName: "Windsurf",
    icon: "windsurf-icon",
    expectedEndpoints: ["api.codeium.com"],
    childProcessTracking: true,
  },
  {
    name: "copilot",
    displayName: "GitHub Copilot",
    icon: "copilot-icon",
    expectedEndpoints: ["api.github.com", "copilot-proxy.githubusercontent.com"],
    childProcessTracking: false,
  },
];

// Default agent status
export const mockAgentStatus: AgentStatus = {
  running: true,
  platform: "linux",
  elevatedPrivileges: false,
  uptimeSeconds: 3600,
  processesTracked: 15,
  eventsCollected: 2500,
};

/**
 * Claude Code session scenario.
 * Simulates a typical Claude Code session with child processes.
 */
export const claudeCodeSession = {
  status: {
    ...mockAgentStatus,
    processesTracked: 5,
    eventsCollected: 150,
  },
  processes: [
    {
      id: "uuid-1",
      pid: 1001,
      ppid: 1000,
      name: "claude",
      cmdline: "claude code --project /home/user/project",
      exePath: "/home/user/.npm/bin/claude",
      agentType: "claude-code",
      startTime: Date.now() - 300000,
      endTime: 0,
      user: "user",
      cwd: "/home/user/project",
    },
    {
      id: "uuid-2",
      pid: 1010,
      ppid: 1001,
      name: "node",
      cmdline: "node /home/user/project/build.js",
      exePath: "/usr/bin/node",
      agentType: "claude-code",
      startTime: Date.now() - 200000,
      endTime: 0,
      user: "user",
      cwd: "/home/user/project",
    },
    {
      id: "uuid-3",
      pid: 1011,
      ppid: 1001,
      name: "git",
      cmdline: "git status",
      exePath: "/usr/bin/git",
      agentType: "claude-code",
      startTime: Date.now() - 150000,
      endTime: Date.now() - 149000,
      user: "user",
      cwd: "/home/user/project",
    },
    {
      id: "uuid-4",
      pid: 1012,
      ppid: 1001,
      name: "bash",
      cmdline: "bash -c npm test",
      exePath: "/bin/bash",
      agentType: "claude-code",
      startTime: Date.now() - 100000,
      endTime: 0,
      user: "user",
      cwd: "/home/user/project",
    },
    {
      id: "uuid-5",
      pid: 1013,
      ppid: 1012,
      name: "npm",
      cmdline: "npm test",
      exePath: "/usr/bin/npm",
      agentType: "claude-code",
      startTime: Date.now() - 99000,
      endTime: 0,
      user: "user",
      cwd: "/home/user/project",
    },
  ] as Process[],
  connections: [
    {
      id: "conn-1",
      processId: "uuid-1",
      pid: 1001,
      protocol: "tcp",
      localAddr: "127.0.0.1",
      localPort: 45678,
      remoteAddr: "104.18.6.192",
      remotePort: 443,
      state: "established",
      timestamp: Date.now() - 280000,
    },
    {
      id: "conn-2",
      processId: "uuid-2",
      pid: 1010,
      protocol: "tcp",
      localAddr: "127.0.0.1",
      localPort: 45679,
      remoteAddr: "registry.npmjs.org",
      remotePort: 443,
      state: "established",
      timestamp: Date.now() - 180000,
    },
  ] as Connection[],
};

/**
 * Cursor IDE session scenario.
 * Simulates Cursor with multiple extensions and language servers.
 */
export const cursorSession = {
  status: {
    ...mockAgentStatus,
    processesTracked: 8,
    eventsCollected: 450,
  },
  processes: [
    {
      id: "uuid-10",
      pid: 2001,
      ppid: 1000,
      name: "Cursor",
      cmdline: "/Applications/Cursor.app/Contents/MacOS/Cursor /home/user/project",
      exePath: "/Applications/Cursor.app/Contents/MacOS/Cursor",
      agentType: "cursor",
      startTime: Date.now() - 600000,
      endTime: 0,
      user: "user",
      cwd: "/home/user/project",
    },
    {
      id: "uuid-11",
      pid: 2002,
      ppid: 2001,
      name: "Cursor Helper",
      cmdline: "Cursor Helper --type=renderer",
      exePath: "/Applications/Cursor.app/Contents/Frameworks/Cursor Helper.app/Contents/MacOS/Cursor Helper",
      agentType: "cursor",
      startTime: Date.now() - 599000,
      endTime: 0,
      user: "user",
      cwd: "/home/user/project",
    },
    {
      id: "uuid-12",
      pid: 2003,
      ppid: 2001,
      name: "node",
      cmdline: "node /home/user/.cursor/extensions/ms-python.python-2024.0.1/pythonFiles/run-jedi-language-server.py",
      exePath: "/usr/bin/node",
      agentType: "cursor",
      startTime: Date.now() - 500000,
      endTime: 0,
      user: "user",
      cwd: "/home/user/project",
    },
    {
      id: "uuid-13",
      pid: 2004,
      ppid: 2001,
      name: "rust-analyzer",
      cmdline: "rust-analyzer",
      exePath: "/home/user/.cargo/bin/rust-analyzer",
      agentType: "cursor",
      startTime: Date.now() - 450000,
      endTime: 0,
      user: "user",
      cwd: "/home/user/project",
    },
    {
      id: "uuid-14",
      pid: 2005,
      ppid: 2001,
      name: "gopls",
      cmdline: "gopls serve",
      exePath: "/home/user/go/bin/gopls",
      agentType: "cursor",
      startTime: Date.now() - 400000,
      endTime: 0,
      user: "user",
      cwd: "/home/user/project",
    },
    {
      id: "uuid-15",
      pid: 2006,
      ppid: 2001,
      name: "typescript-language-server",
      cmdline: "typescript-language-server --stdio",
      exePath: "/home/user/.npm/bin/typescript-language-server",
      agentType: "cursor",
      startTime: Date.now() - 350000,
      endTime: 0,
      user: "user",
      cwd: "/home/user/project",
    },
    {
      id: "uuid-16",
      pid: 2007,
      ppid: 2001,
      name: "git",
      cmdline: "git log --oneline -10",
      exePath: "/usr/bin/git",
      agentType: "cursor",
      startTime: Date.now() - 60000,
      endTime: Date.now() - 59000,
      user: "user",
      cwd: "/home/user/project",
    },
    {
      id: "uuid-17",
      pid: 2008,
      ppid: 2001,
      name: "cargo",
      cmdline: "cargo check",
      exePath: "/home/user/.cargo/bin/cargo",
      agentType: "cursor",
      startTime: Date.now() - 30000,
      endTime: 0,
      user: "user",
      cwd: "/home/user/project",
    },
  ] as Process[],
  connections: [
    {
      id: "conn-10",
      processId: "uuid-10",
      pid: 2001,
      protocol: "tcp",
      localAddr: "127.0.0.1",
      localPort: 55001,
      remoteAddr: "api.cursor.sh",
      remotePort: 443,
      state: "established",
      timestamp: Date.now() - 580000,
    },
    {
      id: "conn-11",
      processId: "uuid-10",
      pid: 2001,
      protocol: "tcp",
      localAddr: "127.0.0.1",
      localPort: 55002,
      remoteAddr: "api.openai.com",
      remotePort: 443,
      state: "established",
      timestamp: Date.now() - 400000,
    },
    {
      id: "conn-12",
      processId: "uuid-17",
      pid: 2008,
      protocol: "tcp",
      localAddr: "127.0.0.1",
      localPort: 55003,
      remoteAddr: "crates.io",
      remotePort: 443,
      state: "established",
      timestamp: Date.now() - 25000,
    },
  ] as Connection[],
};

/**
 * Multi-agent scenario.
 * Simulates multiple AI agents running simultaneously.
 */
export const multiAgentSession = {
  status: {
    ...mockAgentStatus,
    processesTracked: 12,
    eventsCollected: 800,
  },
  processes: [
    // Claude Code processes
    ...claudeCodeSession.processes.map((p, i) => ({
      ...p,
      id: `multi-claude-${i}`,
    })),
    // Cursor processes (subset)
    ...cursorSession.processes.slice(0, 4).map((p, i) => ({
      ...p,
      id: `multi-cursor-${i}`,
      pid: p.pid + 10000,
      ppid: p.ppid === 1000 ? 1000 : p.ppid + 10000,
    })),
    // Aider process
    {
      id: "multi-aider-1",
      pid: 3001,
      ppid: 1000,
      name: "aider",
      cmdline: "aider --model gpt-4 --edit-format diff",
      exePath: "/home/user/.local/bin/aider",
      agentType: "aider",
      startTime: Date.now() - 180000,
      endTime: 0,
      user: "user",
      cwd: "/home/user/other-project",
    },
    {
      id: "multi-aider-2",
      pid: 3002,
      ppid: 3001,
      name: "git",
      cmdline: "git diff HEAD",
      exePath: "/usr/bin/git",
      agentType: "aider",
      startTime: Date.now() - 60000,
      endTime: Date.now() - 55000,
      user: "user",
      cwd: "/home/user/other-project",
    },
    // Windsurf process
    {
      id: "multi-windsurf-1",
      pid: 4001,
      ppid: 1000,
      name: "windsurf",
      cmdline: "windsurf /home/user/third-project",
      exePath: "/opt/windsurf/windsurf",
      agentType: "windsurf",
      startTime: Date.now() - 120000,
      endTime: 0,
      user: "user",
      cwd: "/home/user/third-project",
    },
  ] as Process[],
  connections: [
    ...claudeCodeSession.connections.map((c, i) => ({
      ...c,
      id: `multi-conn-claude-${i}`,
    })),
    ...cursorSession.connections.slice(0, 2).map((c, i) => ({
      ...c,
      id: `multi-conn-cursor-${i}`,
      pid: c.pid + 10000,
    })),
    {
      id: "multi-conn-aider-1",
      processId: "multi-aider-1",
      pid: 3001,
      protocol: "tcp",
      localAddr: "127.0.0.1",
      localPort: 45700,
      remoteAddr: "api.openai.com",
      remotePort: 443,
      state: "established",
      timestamp: Date.now() - 170000,
    },
    {
      id: "multi-conn-windsurf-1",
      processId: "multi-windsurf-1",
      pid: 4001,
      protocol: "tcp",
      localAddr: "127.0.0.1",
      localPort: 45800,
      remoteAddr: "api.codeium.com",
      remotePort: 443,
      state: "established",
      timestamp: Date.now() - 110000,
    },
  ] as Connection[],
};

/**
 * Long-running session scenario.
 * Simulates a 1-hour session with accumulated events.
 */
export const longRunningSession = {
  status: {
    ...mockAgentStatus,
    uptimeSeconds: 3600,
    processesTracked: 50,
    eventsCollected: 5000,
  },
  processes: [
    // Generate a mix of active and exited processes
    ...Array.from({ length: 30 }, (_, i) => ({
      id: `long-proc-${i}`,
      pid: 5000 + i,
      ppid: i === 0 ? 1000 : 5000 + Math.floor(i / 3),
      name: ["node", "git", "cargo", "npm", "python", "bash"][i % 6],
      cmdline: `process-${i} --arg=${i}`,
      exePath: `/usr/bin/process-${i}`,
      agentType: i < 10 ? "claude-code" : i < 20 ? "cursor" : "aider",
      startTime: Date.now() - 3600000 + i * 60000,
      endTime: i % 3 === 0 ? Date.now() - 3600000 + i * 60000 + 30000 : 0,
      user: "user",
      cwd: "/home/user/project",
    })),
  ] as Process[],
  connections: Array.from({ length: 20 }, (_, i) => ({
    id: `long-conn-${i}`,
    processId: `long-proc-${i % 30}`,
    pid: 5000 + (i % 30),
    protocol: i % 3 === 0 ? "udp" : "tcp",
    localAddr: "127.0.0.1",
    localPort: 50000 + i,
    remoteAddr: ["api.anthropic.com", "api.openai.com", "api.cursor.sh", "github.com"][i % 4],
    remotePort: 443,
    state: i % 4 === 0 ? "closed" : "established",
    timestamp: Date.now() - 3600000 + i * 180000,
  })) as Connection[],
};

/**
 * Empty/disconnected state scenario.
 */
export const disconnectedState = {
  status: null,
  processes: [] as Process[],
  connections: [] as Connection[],
};

/**
 * No agents detected scenario.
 */
export const noAgentsSession = {
  status: {
    ...mockAgentStatus,
    processesTracked: 0,
    eventsCollected: 0,
  },
  processes: [] as Process[],
  connections: [] as Connection[],
};
