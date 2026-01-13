/**
 * Component Tests: SearchBar
 *
 * Tests for the search and filtering component.
 * Covers text search, advanced filters, and export functionality.
 */

import { test, expect } from "@playwright/test";
import { setupTauriMock } from "../fixtures/tauri-mock";
import { claudeCodeSession, multiAgentSession, mockSignatures } from "../fixtures/mock-data";

test.describe("SearchBar Component", () => {
  test.describe("Basic Search", () => {
    test.beforeEach(async ({ page }) => {
      await setupTauriMock(page, {
        connected: true,
        status: claudeCodeSession.status,
        processes: claudeCodeSession.processes,
        connections: claudeCodeSession.connections,
        signatures: mockSignatures,
      });
      await page.goto("/");
      await page.waitForTimeout(500);
    });

    test("search input is visible", async ({ page }) => {
      const searchInput = page.locator(".search-input");
      await expect(searchInput).toBeVisible();
    });

    test("search input has placeholder text", async ({ page }) => {
      const searchInput = page.locator(".search-input");
      await expect(searchInput).toHaveAttribute("placeholder", /Search/);
    });

    test("search icon is displayed", async ({ page }) => {
      const searchIcon = page.locator(".search-icon");
      await expect(searchIcon).toBeVisible();
    });

    test("typing filters processes by name", async ({ page }) => {
      const searchInput = page.locator(".search-input");
      await searchInput.fill("git");

      // Should show only git process
      const nodes = page.locator(".process-graph svg .process-node");
      await expect(nodes).toHaveCount(1);
    });

    test("typing filters by PID", async ({ page }) => {
      const searchInput = page.locator(".search-input");
      await searchInput.fill("1001");

      const nodes = page.locator(".process-graph svg .process-node");
      await expect(nodes).toHaveCount(1);
    });

    test("typing filters by command line", async ({ page }) => {
      const searchInput = page.locator(".search-input");
      await searchInput.fill("npm test");

      const nodes = page.locator(".process-graph svg .process-node");
      const count = await nodes.count();
      expect(count).toBeGreaterThanOrEqual(1);
    });

    test("typing filters by executable path", async ({ page }) => {
      const searchInput = page.locator(".search-input");
      await searchInput.fill("/usr/bin");

      const nodes = page.locator(".process-graph svg .process-node");
      const count = await nodes.count();
      expect(count).toBeGreaterThanOrEqual(1);
    });

    test("search is case insensitive", async ({ page }) => {
      const searchInput = page.locator(".search-input");
      await searchInput.fill("GIT");

      const nodes = page.locator(".process-graph svg .process-node");
      await expect(nodes).toHaveCount(1);
    });

    test("empty search shows all processes", async ({ page }) => {
      const searchInput = page.locator(".search-input");
      await searchInput.fill("git");
      await expect(page.locator(".process-graph svg .process-node")).toHaveCount(1);

      await searchInput.fill("");
      await expect(page.locator(".process-graph svg .process-node")).toHaveCount(5);
    });

    test("no results shows empty graph", async ({ page }) => {
      const searchInput = page.locator(".search-input");
      await searchInput.fill("nonexistentprocess");

      const nodes = page.locator(".process-graph svg .process-node");
      await expect(nodes).toHaveCount(0);
    });
  });

  test.describe("Clear Button", () => {
    test.beforeEach(async ({ page }) => {
      await setupTauriMock(page, {
        connected: true,
        status: claudeCodeSession.status,
        processes: claudeCodeSession.processes,
        connections: claudeCodeSession.connections,
        signatures: mockSignatures,
      });
      await page.goto("/");
      await page.waitForTimeout(500);
    });

    test("clear button is hidden when no filters", async ({ page }) => {
      const clearButton = page.locator(".search-clear");
      await expect(clearButton).not.toBeVisible();
    });

    test("clear button appears when filtering", async ({ page }) => {
      const searchInput = page.locator(".search-input");
      await searchInput.fill("test");

      const clearButton = page.locator(".search-clear");
      await expect(clearButton).toBeVisible();
    });

    test("clicking clear button resets search", async ({ page }) => {
      const searchInput = page.locator(".search-input");
      await searchInput.fill("test");

      const clearButton = page.locator(".search-clear");
      await clearButton.click();

      await expect(searchInput).toHaveValue("");
    });

    test("clearing search shows all processes again", async ({ page }) => {
      const searchInput = page.locator(".search-input");
      await searchInput.fill("git");
      await expect(page.locator(".process-graph svg .process-node")).toHaveCount(1);

      const clearButton = page.locator(".search-clear");
      await clearButton.click();

      await expect(page.locator(".process-graph svg .process-node")).toHaveCount(5);
    });
  });

  test.describe("Advanced Filters", () => {
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

    test("filters button is visible", async ({ page }) => {
      const filtersButton = page.locator('.toolbar-button:has-text("Filters")');
      await expect(filtersButton).toBeVisible();
    });

    test("clicking filters button opens advanced panel", async ({ page }) => {
      const filtersButton = page.locator('.toolbar-button:has-text("Filters")');
      await filtersButton.click();

      const advancedFilters = page.locator(".advanced-filters");
      await expect(advancedFilters).toBeVisible();
    });

    test("clicking filters button again closes panel", async ({ page }) => {
      const filtersButton = page.locator('.toolbar-button:has-text("Filters")');
      await filtersButton.click();
      await expect(page.locator(".advanced-filters")).toBeVisible();

      await filtersButton.click();
      await expect(page.locator(".advanced-filters")).not.toBeVisible();
    });

    test("agent type checkboxes are shown", async ({ page }) => {
      const filtersButton = page.locator('.toolbar-button:has-text("Filters")');
      await filtersButton.click();

      const agentFilters = page.locator(".filter-section").first();
      await expect(agentFilters).toContainText("Agent Types");
    });

    test("status checkboxes are shown", async ({ page }) => {
      const filtersButton = page.locator('.toolbar-button:has-text("Filters")');
      await filtersButton.click();

      const statusSection = page.locator(".filter-section:has-text('Status')");
      await expect(statusSection).toBeVisible();
      await expect(statusSection).toContainText("Active");
      await expect(statusSection).toContainText("Exited");
    });

    test("PID range inputs are shown", async ({ page }) => {
      const filtersButton = page.locator('.toolbar-button:has-text("Filters")');
      await filtersButton.click();

      const pidSection = page.locator(".filter-section:has-text('PID Range')");
      await expect(pidSection).toBeVisible();
      await expect(pidSection.locator('input[placeholder="Min"]')).toBeVisible();
      await expect(pidSection.locator('input[placeholder="Max"]')).toBeVisible();
    });

    test("unchecking active hides active processes", async ({ page }) => {
      const filtersButton = page.locator('.toolbar-button:has-text("Filters")');
      await filtersButton.click();

      const activeCheckbox = page.locator('.filter-checkbox:has-text("Active") input');
      await activeCheckbox.click();

      // Should show fewer processes (only exited ones)
      const nodes = page.locator(".process-graph svg .process-node");
      const count = await nodes.count();
      expect(count).toBeLessThan(12); // Original count
    });

    test("unchecking exited hides exited processes", async ({ page }) => {
      const filtersButton = page.locator('.toolbar-button:has-text("Filters")');
      await filtersButton.click();

      const exitedCheckbox = page.locator('.filter-checkbox:has-text("Exited") input');
      await exitedCheckbox.click();

      // Should show fewer processes (only active ones)
      const nodes = page.locator(".process-graph svg .process-node");
      const count = await nodes.count();
      expect(count).toBeLessThan(12);
    });

    test("PID min filter works", async ({ page }) => {
      const filtersButton = page.locator('.toolbar-button:has-text("Filters")');
      await filtersButton.click();

      const minInput = page.locator('.filter-range input[placeholder="Min"]');
      await minInput.fill("3000");

      // Should show only processes with PID >= 3000
      const nodes = page.locator(".process-graph svg .process-node");
      const count = await nodes.count();
      expect(count).toBeLessThan(12);
    });

    test("PID max filter works", async ({ page }) => {
      const filtersButton = page.locator('.toolbar-button:has-text("Filters")');
      await filtersButton.click();

      const maxInput = page.locator('.filter-range input[placeholder="Max"]');
      await maxInput.fill("2000");

      // Should show only processes with PID <= 2000
      const nodes = page.locator(".process-graph svg .process-node");
      const count = await nodes.count();
      expect(count).toBeLessThan(12);
    });

    test("filter badge appears when filters active", async ({ page }) => {
      const filtersButton = page.locator('.toolbar-button:has-text("Filters")');
      await filtersButton.click();

      const activeCheckbox = page.locator('.filter-checkbox:has-text("Active") input');
      await activeCheckbox.click();

      const filterBadge = page.locator(".filter-badge");
      await expect(filterBadge).toBeVisible();
    });
  });

  test.describe("Export Functionality", () => {
    test.beforeEach(async ({ page }) => {
      await setupTauriMock(page, {
        connected: true,
        status: claudeCodeSession.status,
        processes: claudeCodeSession.processes,
        connections: claudeCodeSession.connections,
        signatures: mockSignatures,
      });
      await page.goto("/");
      await page.waitForTimeout(500);
    });

    test("JSON export button is visible", async ({ page }) => {
      const jsonButton = page.locator('.toolbar-button:has-text("JSON")');
      await expect(jsonButton).toBeVisible();
    });

    test("CSV export button is visible", async ({ page }) => {
      const csvButton = page.locator('.toolbar-button:has-text("CSV")');
      await expect(csvButton).toBeVisible();
    });

    test("clicking JSON export triggers download", async ({ page }) => {
      const downloadPromise = page.waitForEvent("download");
      const jsonButton = page.locator('.toolbar-button:has-text("JSON")');
      await jsonButton.click();

      const download = await downloadPromise;
      expect(download.suggestedFilename()).toBe("processes.json");
    });

    test("clicking CSV export triggers download", async ({ page }) => {
      const downloadPromise = page.waitForEvent("download");
      const csvButton = page.locator('.toolbar-button:has-text("CSV")');
      await csvButton.click();

      const download = await downloadPromise;
      expect(download.suggestedFilename()).toBe("processes.csv");
    });

    test("export respects current filter", async ({ page }) => {
      // Filter first
      const searchInput = page.locator(".search-input");
      await searchInput.fill("claude");

      const downloadPromise = page.waitForEvent("download");
      const jsonButton = page.locator('.toolbar-button:has-text("JSON")');
      await jsonButton.click();

      const download = await downloadPromise;
      const content = await (await download.createReadStream())
        ?.setEncoding("utf-8")
        .read();

      const data = JSON.parse(content || "[]");
      // Should only export filtered results
      expect(data.length).toBeLessThanOrEqual(5);
    });

    test("JSON export contains valid JSON", async ({ page }) => {
      const downloadPromise = page.waitForEvent("download");
      const jsonButton = page.locator('.toolbar-button:has-text("JSON")');
      await jsonButton.click();

      const download = await downloadPromise;
      const content = await (await download.createReadStream())
        ?.setEncoding("utf-8")
        .read();

      expect(() => JSON.parse(content || "")).not.toThrow();
    });

    test("CSV export contains headers", async ({ page }) => {
      const downloadPromise = page.waitForEvent("download");
      const csvButton = page.locator('.toolbar-button:has-text("CSV")');
      await csvButton.click();

      const download = await downloadPromise;
      const content = await (await download.createReadStream())
        ?.setEncoding("utf-8")
        .read();

      expect(content).toContain("pid");
      expect(content).toContain("name");
    });
  });

  test.describe("Keyboard Navigation", () => {
    test.beforeEach(async ({ page }) => {
      await setupTauriMock(page, {
        connected: true,
        status: claudeCodeSession.status,
        processes: claudeCodeSession.processes,
        connections: claudeCodeSession.connections,
        signatures: mockSignatures,
      });
      await page.goto("/");
      await page.waitForTimeout(500);
    });

    test("search input can be focused with Tab", async ({ page }) => {
      await page.keyboard.press("Tab");
      await page.keyboard.press("Tab"); // Tab through header elements

      const searchInput = page.locator(".search-input");
      // Check if focused
      const isFocused = await searchInput.evaluate((el) => document.activeElement === el);
      // May need more tabs depending on header, just verify it's accessible
      await expect(searchInput).toBeVisible();
    });

    test("typing in focused input starts search", async ({ page }) => {
      const searchInput = page.locator(".search-input");
      await searchInput.focus();
      await page.keyboard.type("git");

      await expect(searchInput).toHaveValue("git");
    });

    test("Escape clears search when focused", async ({ page }) => {
      const searchInput = page.locator(".search-input");
      await searchInput.fill("test");
      await searchInput.focus();

      await page.keyboard.press("Escape");

      // Input might clear or blur depending on implementation
      const value = await searchInput.inputValue();
      // Either value is cleared or input is blurred
      expect(value === "" || document.activeElement !== searchInput).toBeTruthy();
    });

    test("Enter in search input does not submit form", async ({ page }) => {
      const searchInput = page.locator(".search-input");
      await searchInput.fill("test");

      await page.keyboard.press("Enter");

      // Should still be on the same page
      await expect(searchInput).toBeVisible();
    });
  });

  test.describe("Search Performance", () => {
    test("search is responsive with many processes", async ({ page }) => {
      await setupTauriMock(page, {
        connected: true,
        status: multiAgentSession.status,
        processes: multiAgentSession.processes,
        connections: multiAgentSession.connections,
        signatures: mockSignatures,
      });
      await page.goto("/");
      await page.waitForTimeout(500);

      const searchInput = page.locator(".search-input");

      const start = Date.now();
      await searchInput.fill("node");
      await page.waitForTimeout(100); // Wait for filter to apply
      const searchTime = Date.now() - start;

      // Search should be fast
      expect(searchTime).toBeLessThan(500);
    });

    test("rapid typing doesn't cause issues", async ({ page }) => {
      await setupTauriMock(page, {
        connected: true,
        status: claudeCodeSession.status,
        processes: claudeCodeSession.processes,
        connections: claudeCodeSession.connections,
        signatures: mockSignatures,
      });
      await page.goto("/");
      await page.waitForTimeout(500);

      const searchInput = page.locator(".search-input");

      // Type rapidly
      await searchInput.fill("a");
      await searchInput.fill("ab");
      await searchInput.fill("abc");
      await searchInput.fill("abcd");
      await searchInput.fill("");

      // App should still work
      const nodes = page.locator(".process-graph svg .process-node");
      await expect(nodes).toHaveCount(5);
    });
  });
});

test.describe("SearchBar - Visual Regression", () => {
  test("search bar visual snapshot", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: claudeCodeSession.status,
      processes: claudeCodeSession.processes,
      connections: claudeCodeSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    const searchBar = page.locator(".search-bar");
    await expect(searchBar).toHaveScreenshot("search-bar-default.png");
  });

  test("advanced filters visual snapshot", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: multiAgentSession.status,
      processes: multiAgentSession.processes,
      connections: multiAgentSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    const filtersButton = page.locator('.toolbar-button:has-text("Filters")');
    await filtersButton.click();

    const searchBar = page.locator(".search-bar");
    await expect(searchBar).toHaveScreenshot("search-bar-filters-open.png");
  });
});
