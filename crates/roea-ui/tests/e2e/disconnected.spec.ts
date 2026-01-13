/**
 * E2E Tests: Disconnected & Edge States
 *
 * Tests the UI behavior when the agent is disconnected or
 * in various edge states.
 */

import { test, expect } from "@playwright/test";
import { setupTauriMock } from "./fixtures/tauri-mock";
import { disconnectedState, noAgentsSession, mockSignatures, claudeCodeSession } from "./fixtures/mock-data";

test.describe("Disconnected State", () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      connected: false,
      status: null,
      processes: [],
      connections: [],
      signatures: mockSignatures,
    });
  });

  test("shows disconnected status", async ({ page }) => {
    await page.goto("/");
    await page.waitForTimeout(500);

    const header = page.locator(".header");
    await expect(header).toContainText(/disconnected|not connected/i);
  });

  test("shows reconnect button prominently", async ({ page }) => {
    await page.goto("/");
    await page.waitForTimeout(500);

    const reconnectBtn = page.locator('button:has-text("Reconnect")');
    await expect(reconnectBtn).toBeVisible();
  });

  test("empty graph is displayed gracefully", async ({ page }) => {
    await page.goto("/");
    await page.waitForTimeout(500);

    const graph = page.locator(".process-graph");
    await expect(graph).toBeVisible();

    // Should show empty state message or empty SVG
    const nodes = page.locator(".process-graph svg .node");
    await expect(nodes).toHaveCount(0);
  });

  test("sidebar shows no active agents", async ({ page }) => {
    await page.goto("/");
    await page.waitForTimeout(500);

    const sidebar = page.locator(".sidebar");
    // Agent entries should show 0 counts
    const agentCounts = await sidebar.locator("[data-agent]").allTextContents();
    expect(agentCounts.every((text) => text.includes("0"))).toBe(true);
  });

  test("stats bar shows zero counts", async ({ page }) => {
    await page.goto("/");
    await page.waitForTimeout(500);

    const statsBar = page.locator(".stats-bar");
    await expect(statsBar).toContainText("0");
  });

  test("search is disabled or shows no results", async ({ page }) => {
    await page.goto("/");
    await page.waitForTimeout(500);

    const searchInput = page.locator('.search-bar input[type="text"]');
    await searchInput.fill("test");

    const nodes = page.locator(".process-graph svg .node");
    await expect(nodes).toHaveCount(0);
  });

  test("export buttons handle empty state", async ({ page }) => {
    await page.goto("/");
    await page.waitForTimeout(500);

    const exportBtn = page.locator('button:has-text("Export")');

    // Export should either be disabled or export empty array
    if (await exportBtn.isEnabled()) {
      const downloadPromise = page.waitForEvent("download", { timeout: 2000 }).catch(() => null);
      await exportBtn.click();

      const jsonOption = page.locator('button:has-text("JSON")');
      if (await jsonOption.isVisible()) {
        await jsonOption.click();
        const download = await downloadPromise;
        if (download) {
          const content = await (await download.createReadStream())
            ?.setEncoding("utf-8")
            .read();
          const data = JSON.parse(content || "[]");
          expect(data).toEqual([]);
        }
      }
    }
  });
});

test.describe("No Agents Detected", () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: noAgentsSession.status,
      processes: [],
      connections: [],
      signatures: mockSignatures,
    });
  });

  test("shows connected but no agents message", async ({ page }) => {
    await page.goto("/");
    await page.waitForTimeout(500);

    const header = page.locator(".header");
    await expect(header).toContainText(/connected/i);

    // Graph area might show "no agents" message
    const emptyState = page.locator(".empty-state, .no-agents, .process-graph");
    await expect(emptyState).toBeVisible();
  });

  test("stats show zero processes", async ({ page }) => {
    await page.goto("/");
    await page.waitForTimeout(500);

    const statsBar = page.locator(".stats-bar");
    await expect(statsBar).toContainText("0");
  });

  test("sidebar shows agents with zero counts", async ({ page }) => {
    await page.goto("/");
    await page.waitForTimeout(500);

    const sidebar = page.locator(".sidebar");
    const claudeAgent = sidebar.locator('[data-agent="claude-code"]');

    await expect(claudeAgent).toBeVisible();
    await expect(claudeAgent).toContainText("0");
  });
});

test.describe("Connection Recovery", () => {
  test("reconnect updates UI when successful", async ({ page }) => {
    // Start disconnected
    await setupTauriMock(page, {
      connected: false,
      status: null,
      processes: [],
      connections: [],
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    // Verify disconnected
    await expect(page.locator(".header")).toContainText(/disconnected|not connected/i);

    // Now simulate reconnection by updating mock
    // In real test, clicking reconnect would trigger this
    await setupTauriMock(page, {
      connected: true,
      status: claudeCodeSession.status,
      processes: claudeCodeSession.processes,
      connections: claudeCodeSession.connections,
      signatures: mockSignatures,
    });

    // Reload to apply new mock
    await page.reload();
    await page.waitForTimeout(500);

    // Should now show connected
    await expect(page.locator(".header")).toContainText(/connected/i);
    await expect(page.locator(".process-graph svg .node")).toHaveCount(5);
  });
});

test.describe("Error Handling", () => {
  test("handles API errors gracefully", async ({ page }) => {
    // Set up a mock that will throw errors
    await page.addInitScript(() => {
      const mockInvoke = async (cmd: string): Promise<unknown> => {
        if (cmd === "connect_to_agent") return true;
        if (cmd === "get_signatures") return [];
        throw new Error("API Error: Service unavailable");
      };

      (window as Window & { __TAURI_INTERNALS__: { invoke: typeof mockInvoke } }).__TAURI_INTERNALS__ = {
        invoke: mockInvoke,
      };
    });

    await page.goto("/");
    await page.waitForTimeout(1000);

    // App should not crash, should show error state
    const app = page.locator(".app");
    await expect(app).toBeVisible();
  });

  test("handles malformed data gracefully", async ({ page }) => {
    await page.addInitScript(() => {
      const mockInvoke = async (cmd: string): Promise<unknown> => {
        switch (cmd) {
          case "connect_to_agent":
            return true;
          case "get_status":
            return { running: true, platform: "linux" }; // Missing fields
          case "get_processes":
            return [{ id: "1", pid: null, name: null }]; // Malformed
          case "get_connections":
            return [];
          case "get_signatures":
            return [];
          default:
            return null;
        }
      };

      (window as Window & { __TAURI_INTERNALS__: { invoke: typeof mockInvoke } }).__TAURI_INTERNALS__ = {
        invoke: mockInvoke,
      };
    });

    await page.goto("/");
    await page.waitForTimeout(1000);

    // App should handle gracefully
    const app = page.locator(".app");
    await expect(app).toBeVisible();
  });
});

test.describe("Disconnected - Golden File Tests", () => {
  test("disconnected state snapshot", async ({ page }) => {
    await setupTauriMock(page, {
      connected: false,
      status: null,
      processes: [],
      connections: [],
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    const headerText = await page.locator(".header").textContent();
    expect(headerText).toMatchSnapshot("disconnected-header");
  });

  test("no agents state snapshot", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: noAgentsSession.status,
      processes: [],
      connections: [],
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    const statsText = await page.locator(".stats-bar").textContent();
    expect(statsText).toMatchSnapshot("no-agents-stats");
  });
});
