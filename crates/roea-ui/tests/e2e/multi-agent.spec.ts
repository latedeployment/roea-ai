/**
 * E2E Tests: Multi-Agent Session
 *
 * Tests the UI behavior when multiple AI agents are running
 * simultaneously (Claude Code + Cursor + Aider + Windsurf).
 */

import { test, expect } from "@playwright/test";
import { setupTauriMock } from "./fixtures/tauri-mock";
import { multiAgentSession, mockSignatures } from "./fixtures/mock-data";

test.describe("Multi-Agent Session", () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: multiAgentSession.status,
      processes: multiAgentSession.processes,
      connections: multiAgentSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);
  });

  test("sidebar shows all active agents", async ({ page }) => {
    const sidebar = page.locator(".sidebar");

    // All agents should be visible
    await expect(sidebar.locator('[data-agent="claude-code"]')).toBeVisible();
    await expect(sidebar.locator('[data-agent="cursor"]')).toBeVisible();
    await expect(sidebar.locator('[data-agent="aider"]')).toBeVisible();
    await expect(sidebar.locator('[data-agent="windsurf"]')).toBeVisible();
  });

  test("sidebar shows correct process counts per agent", async ({ page }) => {
    const sidebar = page.locator(".sidebar");

    // Claude Code: 5 processes
    const claudeAgent = sidebar.locator('[data-agent="claude-code"]');
    await expect(claudeAgent).toContainText("5");

    // Cursor: 4 processes (subset from cursorSession)
    const cursorAgent = sidebar.locator('[data-agent="cursor"]');
    await expect(cursorAgent).toContainText("4");

    // Aider: 2 processes
    const aiderAgent = sidebar.locator('[data-agent="aider"]');
    await expect(aiderAgent).toContainText("2");

    // Windsurf: 1 process
    const windsurfAgent = sidebar.locator('[data-agent="windsurf"]');
    await expect(windsurfAgent).toContainText("1");
  });

  test("graph shows all processes from all agents", async ({ page }) => {
    const nodes = page.locator(".process-graph svg .node");
    // 5 (Claude) + 4 (Cursor) + 2 (Aider) + 1 (Windsurf) = 12
    await expect(nodes).toHaveCount(12);
  });

  test("filtering by agent shows only that agent processes", async ({ page }) => {
    // Filter to Claude Code only
    const claudeAgent = page.locator('.sidebar [data-agent="claude-code"]');
    await claudeAgent.click();

    const nodes = page.locator(".process-graph svg .node");
    await expect(nodes).toHaveCount(5);

    // All nodes should be claude-code type
    const nodeTypes = await nodes.evaluateAll((elements) =>
      elements.map((el) => el.getAttribute("data-agent-type"))
    );
    expect(nodeTypes.every((type) => type === "claude-code")).toBe(true);
  });

  test("switching agent filter updates graph", async ({ page }) => {
    // Start with Claude
    const claudeAgent = page.locator('.sidebar [data-agent="claude-code"]');
    await claudeAgent.click();
    await expect(page.locator(".process-graph svg .node")).toHaveCount(5);

    // Switch to Aider
    const aiderAgent = page.locator('.sidebar [data-agent="aider"]');
    await aiderAgent.click();
    await expect(page.locator(".process-graph svg .node")).toHaveCount(2);

    // Switch to Windsurf
    const windsurfAgent = page.locator('.sidebar [data-agent="windsurf"]');
    await windsurfAgent.click();
    await expect(page.locator(".process-graph svg .node")).toHaveCount(1);
  });

  test("clearing filter shows all agents", async ({ page }) => {
    // Filter first
    const claudeAgent = page.locator('.sidebar [data-agent="claude-code"]');
    await claudeAgent.click();
    await expect(page.locator(".process-graph svg .node")).toHaveCount(5);

    // Click again to clear filter (or click "All" button if exists)
    await claudeAgent.click();
    await expect(page.locator(".process-graph svg .node")).toHaveCount(12);
  });

  test("stats bar shows aggregate counts", async ({ page }) => {
    const statsBar = page.locator(".stats-bar");

    // Total processes: 12
    await expect(statsBar).toContainText("12");

    // Total events: 800
    await expect(statsBar).toContainText("800");

    // Active agents: 4
    await expect(statsBar).toContainText("4");
  });

  test("aider process shows correct API connection", async ({ page }) => {
    // Filter to Aider
    const aiderAgent = page.locator('.sidebar [data-agent="aider"]');
    await aiderAgent.click();

    // Click main aider process
    const aiderNode = page.locator('.node[data-pid="3001"]');
    await aiderNode.click();

    const detailsPanel = page.locator(".details-panel");
    await expect(detailsPanel).toContainText("aider");
    await expect(detailsPanel).toContainText("api.openai.com");
  });

  test("windsurf process shows codeium connection", async ({ page }) => {
    // Filter to Windsurf
    const windsurfAgent = page.locator('.sidebar [data-agent="windsurf"]');
    await windsurfAgent.click();

    const windsurfNode = page.locator('.node[data-pid="4001"]');
    await windsurfNode.click();

    const detailsPanel = page.locator(".details-panel");
    await expect(detailsPanel).toContainText("windsurf");
    await expect(detailsPanel).toContainText("api.codeium.com");
  });

  test("search across all agents works", async ({ page }) => {
    const searchInput = page.locator('.search-bar input[type="text"]');
    await searchInput.fill("git");

    const nodes = page.locator(".process-graph svg .node");
    // Both Claude and Aider have git processes
    await expect(nodes).toHaveCount(2);
  });

  test("search with filter combines correctly", async ({ page }) => {
    // Filter to Claude Code first
    const claudeAgent = page.locator('.sidebar [data-agent="claude-code"]');
    await claudeAgent.click();

    // Then search for node
    const searchInput = page.locator('.search-bar input[type="text"]');
    await searchInput.fill("node");

    const nodes = page.locator(".process-graph svg .node");
    // Only Claude's node process
    await expect(nodes).toHaveCount(1);
  });

  test("export includes all agents when no filter", async ({ page }) => {
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
    expect(data.length).toBe(12);
  });

  test("export respects agent filter", async ({ page }) => {
    // Filter to Aider
    const aiderAgent = page.locator('.sidebar [data-agent="aider"]');
    await aiderAgent.click();

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
    expect(data.length).toBe(2);
    expect(data.every((p: { agentType: string }) => p.agentType === "aider")).toBe(true);
  });

  test("different agent processes have distinct visual styles", async ({ page }) => {
    const graph = page.locator(".process-graph svg");

    // Each agent type should have different styling
    const claudeNode = graph.locator('.node[data-agent-type="claude-code"]').first();
    const cursorNode = graph.locator('.node[data-agent-type="cursor"]').first();

    // Get fill colors (they should be different)
    const claudeColor = await claudeNode.evaluate(
      (el) => getComputedStyle(el.querySelector("circle") || el).fill
    );
    const cursorColor = await cursorNode.evaluate(
      (el) => getComputedStyle(el.querySelector("circle") || el).fill
    );

    expect(claudeColor).not.toBe(cursorColor);
  });

  test("connection lines show across agent processes", async ({ page }) => {
    const links = page.locator(".process-graph svg .link");

    // Multiple parent-child links across all agents
    const linkCount = await links.count();
    expect(linkCount).toBeGreaterThan(8); // At least 8 links
  });
});

test.describe("Multi-Agent - Performance Tests", () => {
  test("graph renders quickly with many processes", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: multiAgentSession.status,
      processes: multiAgentSession.processes,
      connections: multiAgentSession.connections,
      signatures: mockSignatures,
    });

    const start = Date.now();
    await page.goto("/");
    await page.waitForSelector(".process-graph svg .node");
    const loadTime = Date.now() - start;

    // Should load in under 3 seconds
    expect(loadTime).toBeLessThan(3000);
  });

  test("filtering is responsive", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: multiAgentSession.status,
      processes: multiAgentSession.processes,
      connections: multiAgentSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    const start = Date.now();
    const claudeAgent = page.locator('.sidebar [data-agent="claude-code"]');
    await claudeAgent.click();
    await page.waitForSelector('.node[data-agent-type="claude-code"]');
    const filterTime = Date.now() - start;

    // Filtering should be near-instant
    expect(filterTime).toBeLessThan(500);
  });

  test("search is responsive", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: multiAgentSession.status,
      processes: multiAgentSession.processes,
      connections: multiAgentSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    const searchInput = page.locator('.search-bar input[type="text"]');

    const start = Date.now();
    await searchInput.fill("git");
    await page.waitForTimeout(100); // Debounce
    const searchTime = Date.now() - start;

    // Search should be fast
    expect(searchTime).toBeLessThan(500);
  });
});

test.describe("Multi-Agent - Golden File Tests", () => {
  test("multi-agent process list snapshot", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: multiAgentSession.status,
      processes: multiAgentSession.processes,
      connections: multiAgentSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    const processData = await page.evaluate(() => {
      const nodes = document.querySelectorAll(".node");
      return Array.from(nodes)
        .map((node) => ({
          pid: node.getAttribute("data-pid"),
          agent: node.getAttribute("data-agent-type"),
          name: node.textContent?.trim(),
        }))
        .sort((a, b) => {
          if (a.agent !== b.agent) return (a.agent || "").localeCompare(b.agent || "");
          return Number(a.pid) - Number(b.pid);
        });
    });

    expect(processData).toMatchSnapshot("multi-agent-process-list");
  });

  test("multi-agent sidebar snapshot", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: multiAgentSession.status,
      processes: multiAgentSession.processes,
      connections: multiAgentSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    const sidebarData = await page.evaluate(() => {
      const agents = document.querySelectorAll(".sidebar [data-agent]");
      return Array.from(agents).map((agent) => ({
        name: agent.getAttribute("data-agent"),
        text: agent.textContent?.trim(),
      }));
    });

    expect(sidebarData).toMatchSnapshot("multi-agent-sidebar");
  });
});
