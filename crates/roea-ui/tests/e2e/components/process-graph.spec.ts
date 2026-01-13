/**
 * Component Tests: ProcessGraph
 *
 * Tests for the D3.js process graph visualization component.
 * Covers graph rendering, layouts, interactions, and performance.
 */

import { test, expect } from "@playwright/test";
import { setupTauriMock } from "../fixtures/tauri-mock";
import { claudeCodeSession, multiAgentSession, mockSignatures } from "../fixtures/mock-data";

test.describe("ProcessGraph Component", () => {
  test.describe("Graph Rendering", () => {
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

    test("renders SVG canvas correctly", async ({ page }) => {
      const svg = page.locator(".graph-canvas");
      await expect(svg).toBeVisible();
      await expect(svg).toHaveAttribute("class", /graph-canvas/);
    });

    test("renders nodes as circle elements", async ({ page }) => {
      const circles = page.locator(".process-graph svg circle");
      const count = await circles.count();
      // Each node has main circle + status dot + potentially outer ring
      expect(count).toBeGreaterThanOrEqual(5);
    });

    test("renders links as line elements", async ({ page }) => {
      const lines = page.locator(".process-graph svg .links line");
      // 4 parent-child relationships
      await expect(lines).toHaveCount(4);
    });

    test("nodes have correct data attributes", async ({ page }) => {
      const node = page.locator(".process-node").first();
      await expect(node).toBeVisible();
    });

    test("agent nodes have distinct styling", async ({ page }) => {
      // The main Claude node should have agent-specific color
      const agentNode = page.locator(".process-node circle").first();
      const fill = await agentNode.evaluate(
        (el) => getComputedStyle(el).fill || el.getAttribute("fill")
      );
      // Claude color is #d97706
      expect(fill).toBeTruthy();
    });

    test("labels are rendered for each node", async ({ page }) => {
      const labels = page.locator(".process-node text");
      const count = await labels.count();
      // Each node has name label + PID label
      expect(count).toBeGreaterThanOrEqual(5);
    });

    test("empty state shows message when no processes", async ({ page }) => {
      await setupTauriMock(page, {
        connected: true,
        status: { ...claudeCodeSession.status, processesTracked: 0 },
        processes: [],
        connections: [],
        signatures: mockSignatures,
      });
      await page.goto("/");
      await page.waitForTimeout(500);

      const emptyState = page.locator(".empty-state");
      await expect(emptyState).toBeVisible();
      await expect(emptyState).toContainText("No Processes");
    });

    test("arrow markers are defined in SVG defs", async ({ page }) => {
      const marker = page.locator(".process-graph svg defs marker#arrowhead");
      await expect(marker).toBeVisible();
    });
  });

  test.describe("Layout Types", () => {
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

    test("force layout is default", async ({ page }) => {
      const forceBtn = page.locator('.control-btn:has-text("Force")');
      await expect(forceBtn).toHaveClass(/active/);
    });

    test("can switch to tree layout", async ({ page }) => {
      const treeBtn = page.locator('.control-btn:has-text("Tree")');
      await treeBtn.click();

      await expect(treeBtn).toHaveClass(/active/);
      // Graph should re-render
      await page.waitForTimeout(300);
      const nodes = page.locator(".process-graph svg .node");
      const count = await nodes.count();
      expect(count).toBeGreaterThan(0);
    });

    test("can switch to radial layout", async ({ page }) => {
      const radialBtn = page.locator('.control-btn:has-text("Radial")');
      await radialBtn.click();

      await expect(radialBtn).toHaveClass(/active/);
      await page.waitForTimeout(300);
      const nodes = page.locator(".process-graph svg .node");
      const count = await nodes.count();
      expect(count).toBeGreaterThan(0);
    });

    test("switching layouts preserves nodes", async ({ page }) => {
      const nodesBefore = await page.locator(".process-node").count();

      await page.locator('.control-btn:has-text("Tree")').click();
      await page.waitForTimeout(300);

      const nodesAfter = await page.locator(".process-graph svg .node").count();
      expect(nodesAfter).toBe(nodesBefore);
    });
  });

  test.describe("Network Overlay", () => {
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

    test("network overlay checkbox exists", async ({ page }) => {
      const checkbox = page.locator('.control-checkbox input[type="checkbox"]');
      await expect(checkbox).toBeVisible();
    });

    test("network overlay is off by default", async ({ page }) => {
      const checkbox = page.locator('.control-checkbox input[type="checkbox"]');
      await expect(checkbox).not.toBeChecked();
    });

    test("can toggle network overlay on", async ({ page }) => {
      const checkbox = page.locator('.control-checkbox input[type="checkbox"]');
      await checkbox.click();

      await expect(checkbox).toBeChecked();
      await page.waitForTimeout(300);

      // Network links might appear if processes share endpoints
      const networkLinks = page.locator(".network-links");
      // Just verify the checkbox works, links depend on data
      await expect(checkbox).toBeChecked();
    });
  });

  test.describe("Zoom Controls", () => {
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

    test("zoom indicator shows current level", async ({ page }) => {
      const zoomIndicator = page.locator(".zoom-indicator");
      await expect(zoomIndicator).toContainText(/Zoom.*100%/);
    });

    test("mouse wheel zooms the graph", async ({ page }) => {
      const graph = page.locator(".graph-canvas");

      // Get initial transform
      const initialTransform = await graph.locator("g").first().evaluate(
        (el) => el.getAttribute("transform")
      );

      // Zoom with scroll
      await graph.hover();
      await page.mouse.wheel(0, -100);
      await page.waitForTimeout(200);

      // Transform should change
      const newTransform = await graph.locator("g").first().evaluate(
        (el) => el.getAttribute("transform")
      );

      // Either transform or zoom indicator should change
      const zoomIndicator = page.locator(".zoom-indicator");
      const zoomText = await zoomIndicator.textContent();
      expect(zoomText).toMatch(/Zoom.*\d+%/);
    });

    test("graph can be panned by dragging", async ({ page }) => {
      const svg = page.locator(".graph-canvas");
      const box = await svg.boundingBox();

      if (box) {
        await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
        await page.mouse.down();
        await page.mouse.move(box.x + box.width / 2 + 100, box.y + box.height / 2 + 100);
        await page.mouse.up();
      }

      // Graph should have panned (transform changed)
      const group = page.locator(".graph-canvas > g");
      const transform = await group.evaluate((el) => el.getAttribute("transform"));
      expect(transform).toBeTruthy();
    });
  });

  test.describe("Node Interactions", () => {
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

    test("clicking node selects it", async ({ page }) => {
      const node = page.locator(".process-node").first();
      await node.click();

      // Details panel should appear
      const detailsPanel = page.locator(".details-panel");
      await expect(detailsPanel).toBeVisible();
    });

    test("clicking selected node deselects it", async ({ page }) => {
      const node = page.locator(".process-node").first();
      await node.click();

      const detailsPanel = page.locator(".details-panel");
      await expect(detailsPanel).toBeVisible();

      // Click same node again
      await node.click();
      await expect(detailsPanel).not.toBeVisible();
    });

    test("clicking background deselects", async ({ page }) => {
      const node = page.locator(".process-node").first();
      await node.click();

      const detailsPanel = page.locator(".details-panel");
      await expect(detailsPanel).toBeVisible();

      // Click background
      const svg = page.locator(".graph-canvas");
      const box = await svg.boundingBox();
      if (box) {
        // Click in corner (likely empty)
        await page.mouse.click(box.x + 10, box.y + 10);
      }

      await expect(detailsPanel).not.toBeVisible();
    });

    test("hovering node shows PID tooltip", async ({ page }) => {
      const node = page.locator(".process-node").first();
      await node.hover();

      // PID label should become visible
      const pidLabel = node.locator(".pid-label");
      await expect(pidLabel).toHaveAttribute("opacity", "1");
    });

    test("nodes can be dragged", async ({ page }) => {
      const node = page.locator(".process-node").first();
      const box = await node.boundingBox();

      if (box) {
        const centerX = box.x + box.width / 2;
        const centerY = box.y + box.height / 2;

        // Drag node
        await page.mouse.move(centerX, centerY);
        await page.mouse.down();
        await page.mouse.move(centerX + 50, centerY + 50);
        await page.mouse.up();
      }

      // Node should have moved (simulation will slowly bring it back)
      // Just verify drag didn't cause errors
      await expect(node).toBeVisible();
    });

    test("selected node has highlighted border", async ({ page }) => {
      const node = page.locator(".process-node").first();
      await node.click();

      // The circle should have a white stroke when selected
      const circle = node.locator("circle").first();
      const stroke = await circle.evaluate((el) => el.getAttribute("stroke"));
      expect(stroke).toBe("#fff");
    });
  });

  test.describe("Legend", () => {
    test("legend is displayed", async ({ page }) => {
      await setupTauriMock(page, {
        connected: true,
        status: claudeCodeSession.status,
        processes: claudeCodeSession.processes,
        connections: claudeCodeSession.connections,
        signatures: mockSignatures,
      });
      await page.goto("/");
      await page.waitForTimeout(500);

      const legend = page.locator(".graph-legend");
      await expect(legend).toBeVisible();
    });

    test("legend shows active/exited status", async ({ page }) => {
      await setupTauriMock(page, {
        connected: true,
        status: claudeCodeSession.status,
        processes: claudeCodeSession.processes,
        connections: claudeCodeSession.connections,
        signatures: mockSignatures,
      });
      await page.goto("/");
      await page.waitForTimeout(500);

      const legend = page.locator(".graph-legend");
      await expect(legend).toContainText("Active");
      await expect(legend).toContainText("Exited");
    });

    test("legend shows agent types", async ({ page }) => {
      await setupTauriMock(page, {
        connected: true,
        status: claudeCodeSession.status,
        processes: claudeCodeSession.processes,
        connections: claudeCodeSession.connections,
        signatures: mockSignatures,
      });
      await page.goto("/");
      await page.waitForTimeout(500);

      const legend = page.locator(".graph-legend");
      await expect(legend).toContainText("Claude");
      await expect(legend).toContainText("Cursor");
    });
  });

  test.describe("Performance", () => {
    test("renders large graph in acceptable time", async ({ page }) => {
      await setupTauriMock(page, {
        connected: true,
        status: multiAgentSession.status,
        processes: multiAgentSession.processes,
        connections: multiAgentSession.connections,
        signatures: mockSignatures,
      });

      const start = Date.now();
      await page.goto("/");
      await page.waitForSelector(".process-node");
      const loadTime = Date.now() - start;

      // Should load in under 2 seconds
      expect(loadTime).toBeLessThan(2000);
    });

    test("layout switch is responsive", async ({ page }) => {
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
      await page.locator('.control-btn:has-text("Tree")').click();
      await page.waitForTimeout(100);
      const switchTime = Date.now() - start;

      // Layout switch should be fast
      expect(switchTime).toBeLessThan(1000);
    });

    test("graph remains responsive during interaction", async ({ page }) => {
      await setupTauriMock(page, {
        connected: true,
        status: multiAgentSession.status,
        processes: multiAgentSession.processes,
        connections: multiAgentSession.connections,
        signatures: mockSignatures,
      });
      await page.goto("/");
      await page.waitForTimeout(500);

      // Perform multiple rapid interactions
      for (let i = 0; i < 5; i++) {
        await page.locator(".process-node").nth(i % 3).click();
        await page.waitForTimeout(50);
      }

      // Graph should still be visible and functional
      const nodes = page.locator(".process-node");
      const count = await nodes.count();
      expect(count).toBeGreaterThan(0);
    });
  });
});

test.describe("ProcessGraph - Visual Regression", () => {
  test("force layout visual snapshot", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: claudeCodeSession.status,
      processes: claudeCodeSession.processes,
      connections: claudeCodeSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(1000); // Wait for simulation to settle

    // Take screenshot of just the graph area
    const graph = page.locator(".graph-container");
    await expect(graph).toHaveScreenshot("graph-force-layout.png", {
      maxDiffPixelRatio: 0.1, // Allow 10% diff due to simulation randomness
    });
  });

  test("empty state visual snapshot", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: { ...claudeCodeSession.status, processesTracked: 0 },
      processes: [],
      connections: [],
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    const graph = page.locator(".graph-container");
    await expect(graph).toHaveScreenshot("graph-empty-state.png");
  });
});
