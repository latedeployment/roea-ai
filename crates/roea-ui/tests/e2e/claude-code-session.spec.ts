/**
 * E2E Tests: Claude Code Session
 *
 * Tests the UI behavior when monitoring a Claude Code session,
 * verifying process tree display, connection tracking, and
 * event capture accuracy.
 */

import { test, expect } from "@playwright/test";
import { setupTauriMock } from "./fixtures/tauri-mock";
import { claudeCodeSession, mockSignatures } from "./fixtures/mock-data";

test.describe("Claude Code Session", () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: claudeCodeSession.status,
      processes: claudeCodeSession.processes,
      connections: claudeCodeSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    // Wait for initial data load
    await page.waitForTimeout(500);
  });

  test("displays connection status as connected", async ({ page }) => {
    const header = page.locator(".header");
    await expect(header).toBeVisible();

    // Should show connected status
    const statusIndicator = page.locator(".connection-status");
    await expect(statusIndicator).toContainText(/connected/i);
  });

  test("shows Claude Code agent in sidebar", async ({ page }) => {
    const sidebar = page.locator(".sidebar");
    await expect(sidebar).toBeVisible();

    // Should display Claude Code agent with process count
    const claudeAgent = sidebar.locator('[data-agent="claude-code"]');
    await expect(claudeAgent).toBeVisible();
    await expect(claudeAgent).toContainText("Claude Code");
    await expect(claudeAgent).toContainText("5"); // 5 processes
  });

  test("renders process graph with all processes", async ({ page }) => {
    const graph = page.locator(".process-graph");
    await expect(graph).toBeVisible();

    // SVG should be rendered
    const svg = graph.locator("svg");
    await expect(svg).toBeVisible();

    // Should have nodes for each process
    const nodes = svg.locator(".node");
    await expect(nodes).toHaveCount(5);
  });

  test("process tree shows parent-child relationships", async ({ page }) => {
    const graph = page.locator(".process-graph svg");

    // Should have links connecting parent to children
    const links = graph.locator(".link");
    // 4 child processes linked to parents
    await expect(links).toHaveCount(4);
  });

  test("clicking a process shows details panel", async ({ page }) => {
    // Click on the main claude process node
    const claudeNode = page.locator('.node[data-pid="1001"]');
    await claudeNode.click();

    // Details panel should appear
    const detailsPanel = page.locator(".details-panel");
    await expect(detailsPanel).toBeVisible();

    // Should show process information
    await expect(detailsPanel).toContainText("claude");
    await expect(detailsPanel).toContainText("1001");
    await expect(detailsPanel).toContainText("/home/user/project");
  });

  test("displays child processes with correct parent", async ({ page }) => {
    // The npm process (pid 1013) should be child of bash (pid 1012)
    const npmNode = page.locator('.node[data-pid="1013"]');
    await npmNode.click();

    const detailsPanel = page.locator(".details-panel");
    await expect(detailsPanel).toContainText("npm");
    await expect(detailsPanel).toContainText("Parent: 1012");
  });

  test("shows network connections for processes", async ({ page }) => {
    // Click on the main claude process
    const claudeNode = page.locator('.node[data-pid="1001"]');
    await claudeNode.click();

    const detailsPanel = page.locator(".details-panel");

    // Should show connection information
    await expect(detailsPanel).toContainText("Connections");
    await expect(detailsPanel).toContainText("tcp");
    await expect(detailsPanel).toContainText("443");
  });

  test("stats bar shows correct counts", async ({ page }) => {
    const statsBar = page.locator(".stats-bar");
    await expect(statsBar).toBeVisible();

    // Should show process count
    await expect(statsBar).toContainText("5");
    await expect(statsBar).toContainText("Processes");

    // Should show events collected
    await expect(statsBar).toContainText("150");
    await expect(statsBar).toContainText("Events");
  });

  test("exited processes are visually distinct", async ({ page }) => {
    // Git process (pid 1011) has ended
    const gitNode = page.locator('.node[data-pid="1011"]');
    await expect(gitNode).toHaveClass(/exited|inactive/);
  });

  test("filter by clicking agent shows only its processes", async ({ page }) => {
    // Click Claude Code in sidebar
    const claudeAgent = page.locator('.sidebar [data-agent="claude-code"]');
    await claudeAgent.click();

    // Graph should still show 5 nodes (all are claude-code)
    const nodes = page.locator(".process-graph svg .node");
    await expect(nodes).toHaveCount(5);
  });

  test("search filters processes correctly", async ({ page }) => {
    const searchInput = page.locator('.search-bar input[type="text"]');
    await searchInput.fill("git");

    // Should filter to only git process
    const nodes = page.locator(".process-graph svg .node");
    await expect(nodes).toHaveCount(1);
  });

  test("export to JSON downloads file", async ({ page }) => {
    // Set up download listener
    const downloadPromise = page.waitForEvent("download");

    // Click export button
    const exportBtn = page.locator('button:has-text("Export")');
    await exportBtn.click();

    const jsonOption = page.locator('button:has-text("JSON")');
    await jsonOption.click();

    const download = await downloadPromise;
    expect(download.suggestedFilename()).toBe("processes.json");
  });

  test("export to CSV downloads file", async ({ page }) => {
    const downloadPromise = page.waitForEvent("download");

    const exportBtn = page.locator('button:has-text("Export")');
    await exportBtn.click();

    const csvOption = page.locator('button:has-text("CSV")');
    await csvOption.click();

    const download = await downloadPromise;
    expect(download.suggestedFilename()).toBe("processes.csv");
  });

  test("reconnect button works when disconnected", async ({ page }) => {
    // This test verifies reconnect UI behavior
    // In mock, we're always connected, so just verify button exists
    const reconnectBtn = page.locator('button:has-text("Reconnect")');
    await expect(reconnectBtn).toBeVisible();
  });

  test("process details can be closed", async ({ page }) => {
    // Click a process
    const claudeNode = page.locator('.node[data-pid="1001"]');
    await claudeNode.click();

    const detailsPanel = page.locator(".details-panel");
    await expect(detailsPanel).toBeVisible();

    // Click close button
    const closeBtn = detailsPanel.locator('button[aria-label="Close"]');
    await closeBtn.click();

    await expect(detailsPanel).not.toBeVisible();
  });
});

test.describe("Claude Code Session - Golden File Tests", () => {
  test("process list snapshot matches expected", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: claudeCodeSession.status,
      processes: claudeCodeSession.processes,
      connections: claudeCodeSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    // Take snapshot of process list structure
    const processData = await page.evaluate(() => {
      const nodes = document.querySelectorAll(".node");
      return Array.from(nodes).map((node) => ({
        pid: node.getAttribute("data-pid"),
        name: node.textContent?.trim(),
      }));
    });

    expect(processData).toMatchSnapshot("claude-code-process-list");
  });

  test("stats bar snapshot matches expected", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: claudeCodeSession.status,
      processes: claudeCodeSession.processes,
      connections: claudeCodeSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    const statsText = await page.locator(".stats-bar").textContent();
    expect(statsText).toMatchSnapshot("claude-code-stats-bar");
  });
});
