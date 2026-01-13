/**
 * E2E Tests: Cursor IDE Session
 *
 * Tests the UI behavior when monitoring a Cursor IDE session,
 * focusing on extension tracking, language servers, and
 * multiple helper processes.
 */

import { test, expect } from "@playwright/test";
import { setupTauriMock } from "./fixtures/tauri-mock";
import { cursorSession, mockSignatures } from "./fixtures/mock-data";

test.describe("Cursor IDE Session", () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: cursorSession.status,
      processes: cursorSession.processes,
      connections: cursorSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);
  });

  test("shows Cursor agent in sidebar with correct count", async ({ page }) => {
    const sidebar = page.locator(".sidebar");
    const cursorAgent = sidebar.locator('[data-agent="cursor"]');

    await expect(cursorAgent).toBeVisible();
    await expect(cursorAgent).toContainText("Cursor");
    await expect(cursorAgent).toContainText("8"); // 8 processes
  });

  test("renders all Cursor processes in graph", async ({ page }) => {
    const nodes = page.locator(".process-graph svg .node");
    await expect(nodes).toHaveCount(8);
  });

  test("Cursor Helper process shows as child", async ({ page }) => {
    // Find Cursor Helper node
    const helperNode = page.locator('.node[data-pid="2002"]');
    await helperNode.click();

    const detailsPanel = page.locator(".details-panel");
    await expect(detailsPanel).toContainText("Cursor Helper");
    await expect(detailsPanel).toContainText("Parent: 2001");
  });

  test("language servers are tracked correctly", async ({ page }) => {
    // Rust analyzer
    const rustAnalyzer = page.locator('.node[data-pid="2004"]');
    await rustAnalyzer.click();

    const detailsPanel = page.locator(".details-panel");
    await expect(detailsPanel).toContainText("rust-analyzer");

    // Close and check gopls
    const closeBtn = detailsPanel.locator('button[aria-label="Close"]');
    await closeBtn.click();

    const gopls = page.locator('.node[data-pid="2005"]');
    await gopls.click();
    await expect(detailsPanel).toContainText("gopls");
  });

  test("typescript language server shows correctly", async ({ page }) => {
    const tsServer = page.locator('.node[data-pid="2006"]');
    await tsServer.click();

    const detailsPanel = page.locator(".details-panel");
    await expect(detailsPanel).toContainText("typescript-language-server");
  });

  test("multiple API connections shown", async ({ page }) => {
    // Click main Cursor process
    const cursorNode = page.locator('.node[data-pid="2001"]');
    await cursorNode.click();

    const detailsPanel = page.locator(".details-panel");

    // Should show multiple connections
    await expect(detailsPanel).toContainText("api.cursor.sh");
    await expect(detailsPanel).toContainText("api.openai.com");
  });

  test("cargo check process tracked", async ({ page }) => {
    const cargoNode = page.locator('.node[data-pid="2008"]');
    await cargoNode.click();

    const detailsPanel = page.locator(".details-panel");
    await expect(detailsPanel).toContainText("cargo");
    await expect(detailsPanel).toContainText("cargo check");
  });

  test("git command shown as exited", async ({ page }) => {
    // Git process (pid 2007) has ended
    const gitNode = page.locator('.node[data-pid="2007"]');
    await expect(gitNode).toHaveClass(/exited|inactive/);
  });

  test("stats bar shows Cursor session counts", async ({ page }) => {
    const statsBar = page.locator(".stats-bar");

    await expect(statsBar).toContainText("8"); // processes
    await expect(statsBar).toContainText("450"); // events
  });

  test("extension connections to crates.io tracked", async ({ page }) => {
    // Click cargo process which has crates.io connection
    const cargoNode = page.locator('.node[data-pid="2008"]');
    await cargoNode.click();

    const detailsPanel = page.locator(".details-panel");
    await expect(detailsPanel).toContainText("crates.io");
  });

  test("process tree depth is correctly displayed", async ({ page }) => {
    // Cursor -> Helper, Cursor -> language servers, Cursor -> git/cargo
    // All children should connect to the main Cursor process
    const links = page.locator(".process-graph svg .link");

    // 7 child processes connected to parent (Cursor)
    await expect(links).toHaveCount(7);
  });

  test("search finds rust-analyzer", async ({ page }) => {
    const searchInput = page.locator('.search-bar input[type="text"]');
    await searchInput.fill("rust");

    const nodes = page.locator(".process-graph svg .node");
    await expect(nodes).toHaveCount(1);

    const node = nodes.first();
    await expect(node).toContainText("rust-analyzer");
  });

  test("search finds all language servers", async ({ page }) => {
    const searchInput = page.locator('.search-bar input[type="text"]');
    await searchInput.fill("server");

    const nodes = page.locator(".process-graph svg .node");
    // typescript-language-server
    await expect(nodes).toHaveCount(1);
  });

  test("filtering shows only Cursor processes", async ({ page }) => {
    // Click Cursor in sidebar (all processes are Cursor)
    const cursorAgent = page.locator('.sidebar [data-agent="cursor"]');
    await cursorAgent.click();

    const nodes = page.locator(".process-graph svg .node");
    await expect(nodes).toHaveCount(8);
  });

  test("process command line arguments displayed", async ({ page }) => {
    // Click Cursor Helper
    const helperNode = page.locator('.node[data-pid="2002"]');
    await helperNode.click();

    const detailsPanel = page.locator(".details-panel");
    await expect(detailsPanel).toContainText("--type=renderer");
  });

  test("executable paths displayed correctly", async ({ page }) => {
    // Click main Cursor
    const cursorNode = page.locator('.node[data-pid="2001"]');
    await cursorNode.click();

    const detailsPanel = page.locator(".details-panel");
    await expect(detailsPanel).toContainText("/Applications/Cursor.app");
  });
});

test.describe("Cursor Session - Golden File Tests", () => {
  test("cursor process list snapshot matches expected", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: cursorSession.status,
      processes: cursorSession.processes,
      connections: cursorSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    const processData = await page.evaluate(() => {
      const nodes = document.querySelectorAll(".node");
      return Array.from(nodes)
        .map((node) => ({
          pid: node.getAttribute("data-pid"),
          name: node.textContent?.trim(),
        }))
        .sort((a, b) => Number(a.pid) - Number(b.pid));
    });

    expect(processData).toMatchSnapshot("cursor-process-list");
  });

  test("cursor sidebar snapshot matches expected", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: cursorSession.status,
      processes: cursorSession.processes,
      connections: cursorSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    const sidebarText = await page.locator(".sidebar").textContent();
    expect(sidebarText).toMatchSnapshot("cursor-sidebar");
  });
});
