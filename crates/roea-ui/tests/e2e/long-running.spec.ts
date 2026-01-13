/**
 * E2E Tests: Long-Running Session
 *
 * Tests the UI stability and behavior during extended sessions
 * with accumulated processes and events.
 */

import { test, expect } from "@playwright/test";
import { setupTauriMock } from "./fixtures/tauri-mock";
import { longRunningSession, mockSignatures } from "./fixtures/mock-data";

test.describe("Long-Running Session", () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: longRunningSession.status,
      processes: longRunningSession.processes,
      connections: longRunningSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);
  });

  test("displays large number of processes", async ({ page }) => {
    const nodes = page.locator(".process-graph svg .node");
    // Should have 30 processes
    await expect(nodes).toHaveCount(30);
  });

  test("stats show accumulated counts", async ({ page }) => {
    const statsBar = page.locator(".stats-bar");

    // 50 processes tracked total (from status)
    await expect(statsBar).toContainText("50");

    // 5000 events collected
    await expect(statsBar).toContainText("5000");
  });

  test("uptime shown correctly", async ({ page }) => {
    const header = page.locator(".header");
    // 3600 seconds = 1 hour
    await expect(header).toContainText(/1.*hour|3600.*sec/i);
  });

  test("mix of active and exited processes handled", async ({ page }) => {
    const activeNodes = page.locator('.process-graph svg .node:not(.exited):not(.inactive)');
    const exitedNodes = page.locator('.process-graph svg .node.exited, .process-graph svg .node.inactive');

    // Some should be active, some exited
    const activeCount = await activeNodes.count();
    const exitedCount = await exitedNodes.count();

    expect(activeCount).toBeGreaterThan(0);
    expect(exitedCount).toBeGreaterThan(0);
  });

  test("search performs well with many processes", async ({ page }) => {
    const searchInput = page.locator('.search-bar input[type="text"]');

    const start = Date.now();
    await searchInput.fill("node");
    await page.waitForTimeout(200); // Wait for filter to apply
    const searchTime = Date.now() - start;

    // Should still be responsive
    expect(searchTime).toBeLessThan(1000);
  });

  test("graph renders without performance issues", async ({ page }) => {
    // Force re-render by resizing
    await page.setViewportSize({ width: 800, height: 600 });
    await page.waitForTimeout(100);
    await page.setViewportSize({ width: 1200, height: 800 });

    // Graph should still be responsive
    const svg = page.locator(".process-graph svg");
    await expect(svg).toBeVisible();

    const nodes = svg.locator(".node");
    await expect(nodes).toHaveCount(30);
  });

  test("multiple agents tracked over time", async ({ page }) => {
    const sidebar = page.locator(".sidebar");

    // Should show all agents with counts
    await expect(sidebar.locator('[data-agent="claude-code"]')).toBeVisible();
    await expect(sidebar.locator('[data-agent="cursor"]')).toBeVisible();
    await expect(sidebar.locator('[data-agent="aider"]')).toBeVisible();
  });

  test("closed connections handled correctly", async ({ page }) => {
    // Some connections are in closed state
    // Click on a process with closed connection
    const node = page.locator('.node[data-pid="5000"]');
    await node.click();

    const detailsPanel = page.locator(".details-panel");
    // Connection list should handle closed state gracefully
    await expect(detailsPanel).toBeVisible();
  });

  test("export handles large dataset", async ({ page }) => {
    const downloadPromise = page.waitForEvent("download");

    const exportBtn = page.locator('button:has-text("Export")');
    await exportBtn.click();

    const jsonOption = page.locator('button:has-text("JSON")');
    await jsonOption.click();

    const download = await downloadPromise;
    const content = await (await download.createReadStream())
      ?.setEncoding("utf-8")
      .read();

    const data = JSON.parse(content || "[]");
    expect(data.length).toBe(30);
  });

  test("tree depth handles complex relationships", async ({ page }) => {
    // Processes are organized in groups of 3 (parent with 2-3 children)
    const links = page.locator(".process-graph svg .link");
    const linkCount = await links.count();

    // Should have many parent-child links
    expect(linkCount).toBeGreaterThan(20);
  });
});

test.describe("Long-Running - Stability Tests", () => {
  test("rapid filtering does not cause issues", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: longRunningSession.status,
      processes: longRunningSession.processes,
      connections: longRunningSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    const sidebar = page.locator(".sidebar");

    // Rapidly switch between agents
    for (let i = 0; i < 10; i++) {
      await sidebar.locator('[data-agent="claude-code"]').click();
      await page.waitForTimeout(50);
      await sidebar.locator('[data-agent="cursor"]').click();
      await page.waitForTimeout(50);
      await sidebar.locator('[data-agent="aider"]').click();
      await page.waitForTimeout(50);
    }

    // App should still be responsive
    const nodes = page.locator(".process-graph svg .node");
    const count = await nodes.count();
    expect(count).toBeGreaterThan(0);
  });

  test("rapid search does not cause issues", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: longRunningSession.status,
      processes: longRunningSession.processes,
      connections: longRunningSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    const searchInput = page.locator('.search-bar input[type="text"]');

    // Rapid search changes
    const searchTerms = ["node", "git", "cargo", "npm", "python", "bash", ""];
    for (const term of searchTerms) {
      await searchInput.fill(term);
      await page.waitForTimeout(50);
    }

    // App should still be responsive
    const nodes = page.locator(".process-graph svg .node");
    const count = await nodes.count();
    expect(count).toBeGreaterThan(0);
  });

  test("details panel works under load", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: longRunningSession.status,
      processes: longRunningSession.processes,
      connections: longRunningSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    // Click multiple processes rapidly
    const nodes = page.locator(".process-graph svg .node");
    const count = await nodes.count();

    for (let i = 0; i < Math.min(count, 5); i++) {
      await nodes.nth(i).click();
      await page.waitForTimeout(100);
    }

    // Details panel should work
    const detailsPanel = page.locator(".details-panel");
    await expect(detailsPanel).toBeVisible();
  });
});
