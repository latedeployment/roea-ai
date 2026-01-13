/**
 * Tauri IPC mock for E2E testing.
 *
 * This module provides utilities to mock Tauri's invoke function,
 * allowing E2E tests to run without the actual Rust backend.
 */

import type { Page } from "@playwright/test";
import type { Process, Connection, AgentSignature, AgentStatus } from "../../../src/lib/types";

export interface MockState {
  connected: boolean;
  status: AgentStatus | null;
  processes: Process[];
  connections: Connection[];
  signatures: AgentSignature[];
}

/**
 * Default signatures used when no custom ones are provided.
 */
const defaultSignatures: AgentSignature[] = [
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
];

/**
 * Initialize Tauri mock on a Playwright page.
 *
 * This replaces the Tauri invoke function with a mock that returns
 * data from the provided state.
 */
export async function setupTauriMock(page: Page, initialState: Partial<MockState> = {}): Promise<void> {
  const state: MockState = {
    connected: initialState.connected ?? true,
    status: initialState.status ?? null,
    processes: initialState.processes ?? [],
    connections: initialState.connections ?? [],
    signatures: initialState.signatures ?? defaultSignatures,
  };

  // Inject mock before page loads
  await page.addInitScript((mockState: MockState) => {
    // Create a mock invoke function
    const mockInvoke = async (cmd: string, _args?: Record<string, unknown>): Promise<unknown> => {
      // Simulate network delay
      await new Promise((resolve) => setTimeout(resolve, 50));

      switch (cmd) {
        case "connect_to_agent":
          return mockState.connected;

        case "get_status":
          if (!mockState.connected || !mockState.status) {
            throw new Error("Not connected to agent");
          }
          return mockState.status;

        case "get_processes":
          if (!mockState.connected) {
            throw new Error("Not connected to agent");
          }
          return mockState.processes;

        case "get_connections":
          if (!mockState.connected) {
            throw new Error("Not connected to agent");
          }
          return mockState.connections;

        case "get_signatures":
          if (!mockState.connected) {
            throw new Error("Not connected to agent");
          }
          return mockState.signatures;

        default:
          console.warn(`Unknown Tauri command: ${cmd}`);
          return null;
      }
    };

    // Override the Tauri API
    (window as Window & { __TAURI_INTERNALS__: { invoke: typeof mockInvoke } }).__TAURI_INTERNALS__ = {
      invoke: mockInvoke,
    };

    // Also mock @tauri-apps/api/core
    Object.defineProperty(window, "__TAURI_INVOKE__", {
      value: mockInvoke,
      writable: false,
    });
  }, state);
}

/**
 * Update mock state during a test.
 *
 * Useful for simulating dynamic changes like new processes appearing.
 */
export async function updateMockState(page: Page, updates: Partial<MockState>): Promise<void> {
  await page.evaluate((newState) => {
    const tauriInternals = (window as Window & { __TAURI_MOCK_STATE__?: MockState }).__TAURI_MOCK_STATE__;
    if (tauriInternals) {
      Object.assign(tauriInternals, newState);
    }
  }, updates);
}

/**
 * Create a page with Tauri mock configured.
 */
export async function createMockedPage(
  page: Page,
  scenario: {
    status?: AgentStatus | null;
    processes?: Process[];
    connections?: Connection[];
    connected?: boolean;
  }
): Promise<Page> {
  await setupTauriMock(page, {
    connected: scenario.connected ?? true,
    status: scenario.status ?? null,
    processes: scenario.processes ?? [],
    connections: scenario.connections ?? [],
  });

  return page;
}
