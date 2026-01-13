/**
 * Component Tests: Responsive Design
 *
 * Tests for responsive layout behavior across different screen sizes.
 */

import { test, expect } from "@playwright/test";
import { setupTauriMock } from "../fixtures/tauri-mock";
import { claudeCodeSession, mockSignatures } from "../fixtures/mock-data";

// Common viewport sizes
const viewports = {
  mobile: { width: 375, height: 667 },
  tablet: { width: 768, height: 1024 },
  desktop: { width: 1280, height: 800 },
  widescreen: { width: 1920, height: 1080 },
  minimum: { width: 800, height: 600 }, // App minimum size
};

test.describe("Responsive Design", () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: claudeCodeSession.status,
      processes: claudeCodeSession.processes,
      connections: claudeCodeSession.connections,
      signatures: mockSignatures,
    });
  });

  test.describe("Desktop Viewport", () => {
    test.beforeEach(async ({ page }) => {
      await page.setViewportSize(viewports.desktop);
      await page.goto("/");
      await page.waitForTimeout(500);
    });

    test("full layout displays correctly", async ({ page }) => {
      // All main components should be visible
      await expect(page.locator(".header")).toBeVisible();
      await expect(page.locator(".sidebar")).toBeVisible();
      await expect(page.locator(".process-graph")).toBeVisible();
      await expect(page.locator(".stats-bar")).toBeVisible();
    });

    test("sidebar is visible", async ({ page }) => {
      const sidebar = page.locator(".sidebar");
      await expect(sidebar).toBeVisible();

      const box = await sidebar.boundingBox();
      expect(box?.width).toBeGreaterThan(150);
    });

    test("graph has sufficient space", async ({ page }) => {
      const graph = page.locator(".graph-container");
      const box = await graph.boundingBox();

      expect(box?.width).toBeGreaterThan(500);
      expect(box?.height).toBeGreaterThan(400);
    });

    test("search bar is fully visible", async ({ page }) => {
      const searchBar = page.locator(".search-bar");
      await expect(searchBar).toBeVisible();

      // All search elements should be visible
      await expect(page.locator(".search-input")).toBeVisible();
      await expect(page.locator('.toolbar-button:has-text("JSON")')).toBeVisible();
      await expect(page.locator('.toolbar-button:has-text("CSV")')).toBeVisible();
    });
  });

  test.describe("Widescreen Viewport", () => {
    test.beforeEach(async ({ page }) => {
      await page.setViewportSize(viewports.widescreen);
      await page.goto("/");
      await page.waitForTimeout(500);
    });

    test("layout expands to fill space", async ({ page }) => {
      const app = page.locator(".app");
      const box = await app.boundingBox();

      expect(box?.width).toBe(viewports.widescreen.width);
    });

    test("graph expands proportionally", async ({ page }) => {
      const graph = page.locator(".graph-container");
      const box = await graph.boundingBox();

      expect(box?.width).toBeGreaterThan(1000);
    });

    test("details panel has room when open", async ({ page }) => {
      const node = page.locator(".process-node").first();
      await node.click();

      const detailsPanel = page.locator(".details-panel");
      await expect(detailsPanel).toBeVisible();

      const box = await detailsPanel.boundingBox();
      expect(box?.width).toBeGreaterThan(200);
    });
  });

  test.describe("Minimum Size Viewport", () => {
    test.beforeEach(async ({ page }) => {
      await page.setViewportSize(viewports.minimum);
      await page.goto("/");
      await page.waitForTimeout(500);
    });

    test("app renders at minimum size", async ({ page }) => {
      // All essential elements should still be visible
      await expect(page.locator(".header")).toBeVisible();
      await expect(page.locator(".process-graph")).toBeVisible();
    });

    test("no horizontal scrollbar at minimum width", async ({ page }) => {
      const hasHScroll = await page.evaluate(() => {
        return document.documentElement.scrollWidth > document.documentElement.clientWidth;
      });

      expect(hasHScroll).toBe(false);
    });

    test("graph container adapts to small size", async ({ page }) => {
      const graph = page.locator(".graph-container");
      const box = await graph.boundingBox();

      // Should still have reasonable size
      expect(box?.width).toBeGreaterThan(400);
      expect(box?.height).toBeGreaterThan(300);
    });

    test("controls remain accessible", async ({ page }) => {
      // Layout buttons should still be visible
      await expect(page.locator('.control-btn:has-text("Force")')).toBeVisible();
    });
  });

  test.describe("Tablet Viewport", () => {
    test.beforeEach(async ({ page }) => {
      await page.setViewportSize(viewports.tablet);
      await page.goto("/");
      await page.waitForTimeout(500);
    });

    test("layout adapts to tablet size", async ({ page }) => {
      await expect(page.locator(".header")).toBeVisible();
      await expect(page.locator(".process-graph")).toBeVisible();
    });

    test("sidebar may collapse or remain visible", async ({ page }) => {
      // Sidebar behavior depends on implementation
      const sidebar = page.locator(".sidebar");
      // Just verify app works at this size
      await expect(page.locator(".app")).toBeVisible();
    });

    test("search bar remains functional", async ({ page }) => {
      const searchInput = page.locator(".search-input");
      await expect(searchInput).toBeVisible();

      await searchInput.fill("test");
      await expect(searchInput).toHaveValue("test");
    });
  });

  test.describe("Dynamic Resize", () => {
    test("graph re-renders on viewport resize", async ({ page }) => {
      await page.setViewportSize(viewports.desktop);
      await page.goto("/");
      await page.waitForTimeout(500);

      const initialNodes = await page.locator(".process-node").count();

      // Resize viewport
      await page.setViewportSize(viewports.widescreen);
      await page.waitForTimeout(300);

      // Graph should still work
      const afterNodes = await page.locator(".process-node").count();
      expect(afterNodes).toBe(initialNodes);
    });

    test("layout controls remain visible after resize", async ({ page }) => {
      await page.setViewportSize(viewports.desktop);
      await page.goto("/");
      await page.waitForTimeout(500);

      await page.setViewportSize(viewports.minimum);
      await page.waitForTimeout(300);

      await expect(page.locator('.control-btn:has-text("Force")')).toBeVisible();
    });

    test("no content overflow after multiple resizes", async ({ page }) => {
      await page.setViewportSize(viewports.desktop);
      await page.goto("/");
      await page.waitForTimeout(500);

      // Resize multiple times
      await page.setViewportSize(viewports.widescreen);
      await page.waitForTimeout(100);
      await page.setViewportSize(viewports.tablet);
      await page.waitForTimeout(100);
      await page.setViewportSize(viewports.minimum);
      await page.waitForTimeout(100);
      await page.setViewportSize(viewports.desktop);
      await page.waitForTimeout(300);

      // Check for overflow
      const hasOverflow = await page.evaluate(() => {
        return document.documentElement.scrollWidth > document.documentElement.clientWidth;
      });

      expect(hasOverflow).toBe(false);
    });
  });

  test.describe("Component Layout", () => {
    test("header spans full width", async ({ page }) => {
      await page.setViewportSize(viewports.desktop);
      await page.goto("/");
      await page.waitForTimeout(500);

      const header = page.locator(".header");
      const headerBox = await header.boundingBox();
      const appBox = await page.locator(".app").boundingBox();

      expect(headerBox?.width).toBe(appBox?.width);
    });

    test("stats bar spans full width", async ({ page }) => {
      await page.setViewportSize(viewports.desktop);
      await page.goto("/");
      await page.waitForTimeout(500);

      const statsBar = page.locator(".stats-bar");
      const statsBox = await statsBar.boundingBox();
      const appBox = await page.locator(".app").boundingBox();

      expect(statsBox?.width).toBe(appBox?.width);
    });

    test("main content fills remaining height", async ({ page }) => {
      await page.setViewportSize(viewports.desktop);
      await page.goto("/");
      await page.waitForTimeout(500);

      const mainContent = page.locator(".main-content");
      const mainBox = await mainContent.boundingBox();

      expect(mainBox?.height).toBeGreaterThan(400);
    });
  });

  test.describe("Details Panel Responsive", () => {
    test("details panel fits in desktop view", async ({ page }) => {
      await page.setViewportSize(viewports.desktop);
      await page.goto("/");
      await page.waitForTimeout(500);

      const node = page.locator(".process-node").first();
      await node.click();

      const detailsPanel = page.locator(".details-panel");
      await expect(detailsPanel).toBeVisible();

      const box = await detailsPanel.boundingBox();
      expect(box?.width).toBeLessThan(viewports.desktop.width);
    });

    test("details panel adapts to smaller viewport", async ({ page }) => {
      await page.setViewportSize(viewports.minimum);
      await page.goto("/");
      await page.waitForTimeout(500);

      const node = page.locator(".process-node").first();
      await node.click();

      const detailsPanel = page.locator(".details-panel");
      await expect(detailsPanel).toBeVisible();

      // Panel should still be usable
      const box = await detailsPanel.boundingBox();
      expect(box?.width).toBeGreaterThan(200);
    });
  });
});

test.describe("Responsive - Visual Regression", () => {
  test("desktop viewport snapshot", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: claudeCodeSession.status,
      processes: claudeCodeSession.processes,
      connections: claudeCodeSession.connections,
      signatures: mockSignatures,
    });
    await page.setViewportSize(viewports.desktop);
    await page.goto("/");
    await page.waitForTimeout(1000);

    await expect(page).toHaveScreenshot("layout-desktop.png", {
      maxDiffPixelRatio: 0.1,
    });
  });

  test("minimum viewport snapshot", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: claudeCodeSession.status,
      processes: claudeCodeSession.processes,
      connections: claudeCodeSession.connections,
      signatures: mockSignatures,
    });
    await page.setViewportSize(viewports.minimum);
    await page.goto("/");
    await page.waitForTimeout(1000);

    await expect(page).toHaveScreenshot("layout-minimum.png", {
      maxDiffPixelRatio: 0.1,
    });
  });
});
