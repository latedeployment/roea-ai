/**
 * Performance Tests
 *
 * Tests for UI render performance, load times, and responsiveness.
 * Target: 60fps rendering, sub-second interactions.
 */

import { test, expect } from "@playwright/test";
import { setupTauriMock } from "./fixtures/tauri-mock";
import { claudeCodeSession, multiAgentSession, longRunningSession, mockSignatures } from "./fixtures/mock-data";
import type { Process, Connection } from "../../src/lib/types";

// Generate large datasets for stress testing
function generateLargeDataset(processCount: number, connectionCount: number) {
  const processes: Process[] = Array.from({ length: processCount }, (_, i) => ({
    id: `stress-${i}`,
    pid: 10000 + i,
    ppid: i === 0 ? 1 : 10000 + Math.floor(i / 5),
    name: `process-${i}`,
    cmdline: `process-${i} --stress-test`,
    exePath: `/usr/bin/process-${i}`,
    agentType: i % 10 === 0 ? ["claude-code", "cursor", "aider", "windsurf"][i % 4] : "",
    startTime: Date.now() - i * 1000,
    endTime: i % 5 === 0 ? Date.now() - i * 500 : 0,
    user: "user",
    cwd: "/home/user",
  }));

  const connections: Connection[] = Array.from({ length: connectionCount }, (_, i) => ({
    id: `conn-stress-${i}`,
    processId: `stress-${i % processCount}`,
    pid: 10000 + (i % processCount),
    protocol: i % 3 === 0 ? "udp" : "tcp",
    localAddr: "127.0.0.1",
    localPort: 50000 + i,
    remoteAddr: ["api.anthropic.com", "api.openai.com", "github.com"][i % 3],
    remotePort: 443,
    state: i % 4 === 0 ? "closed" : "established",
    timestamp: Date.now() - i * 100,
  }));

  return { processes, connections };
}

test.describe("Performance - Load Times", () => {
  test("initial page load under 2 seconds", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: claudeCodeSession.status,
      processes: claudeCodeSession.processes,
      connections: claudeCodeSession.connections,
      signatures: mockSignatures,
    });

    const start = Date.now();
    await page.goto("/");
    await page.waitForSelector(".process-graph svg");
    const loadTime = Date.now() - start;

    expect(loadTime).toBeLessThan(2000);
    console.log(`Initial load time: ${loadTime}ms`);
  });

  test("graph renders under 500ms for small dataset", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: claudeCodeSession.status,
      processes: claudeCodeSession.processes,
      connections: claudeCodeSession.connections,
      signatures: mockSignatures,
    });

    await page.goto("/");
    const start = Date.now();
    await page.waitForSelector(".process-node");
    const renderTime = Date.now() - start;

    expect(renderTime).toBeLessThan(500);
    console.log(`Graph render time (5 nodes): ${renderTime}ms`);
  });

  test("graph renders under 1 second for medium dataset", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: multiAgentSession.status,
      processes: multiAgentSession.processes,
      connections: multiAgentSession.connections,
      signatures: mockSignatures,
    });

    await page.goto("/");
    const start = Date.now();
    await page.waitForSelector(".process-node");
    const renderTime = Date.now() - start;

    expect(renderTime).toBeLessThan(1000);
    console.log(`Graph render time (12 nodes): ${renderTime}ms`);
  });

  test("graph renders under 3 seconds for large dataset", async ({ page }) => {
    const { processes, connections } = generateLargeDataset(100, 200);

    await setupTauriMock(page, {
      connected: true,
      status: { ...claudeCodeSession.status, processesTracked: 100, eventsCollected: 1000 },
      processes,
      connections,
      signatures: mockSignatures,
    });

    await page.goto("/");
    const start = Date.now();
    await page.waitForSelector(".process-node", { timeout: 5000 });
    const renderTime = Date.now() - start;

    expect(renderTime).toBeLessThan(3000);
    console.log(`Graph render time (100 nodes): ${renderTime}ms`);
  });
});

test.describe("Performance - Interaction Latency", () => {
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

  test("node click response under 100ms", async ({ page }) => {
    const node = page.locator(".process-node").first();

    const start = Date.now();
    await node.click();
    await page.locator(".details-panel").waitFor({ state: "visible" });
    const responseTime = Date.now() - start;

    expect(responseTime).toBeLessThan(100);
    console.log(`Node click response: ${responseTime}ms`);
  });

  test("search input response under 200ms", async ({ page }) => {
    const searchInput = page.locator(".search-input");

    const start = Date.now();
    await searchInput.fill("git");
    await page.waitForTimeout(50); // Debounce
    const responseTime = Date.now() - start;

    expect(responseTime).toBeLessThan(200);
    console.log(`Search input response: ${responseTime}ms`);
  });

  test("layout switch under 500ms", async ({ page }) => {
    const treeBtn = page.locator('.control-btn:has-text("Tree")');

    const start = Date.now();
    await treeBtn.click();
    await page.waitForTimeout(100);
    const responseTime = Date.now() - start;

    expect(responseTime).toBeLessThan(500);
    console.log(`Layout switch response: ${responseTime}ms`);
  });

  test("filter toggle under 100ms", async ({ page }) => {
    const filtersBtn = page.locator('.toolbar-button:has-text("Filters")');

    const start = Date.now();
    await filtersBtn.click();
    await page.locator(".advanced-filters").waitFor({ state: "visible" });
    const responseTime = Date.now() - start;

    expect(responseTime).toBeLessThan(100);
    console.log(`Filter toggle response: ${responseTime}ms`);
  });

  test("export download under 500ms", async ({ page }) => {
    const downloadPromise = page.waitForEvent("download");

    const start = Date.now();
    await page.locator('.toolbar-button:has-text("JSON")').click();
    await downloadPromise;
    const responseTime = Date.now() - start;

    expect(responseTime).toBeLessThan(500);
    console.log(`Export download response: ${responseTime}ms`);
  });
});

test.describe("Performance - Render Quality", () => {
  test("maintains 60fps during idle", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: claudeCodeSession.status,
      processes: claudeCodeSession.processes,
      connections: claudeCodeSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(1000);

    // Measure frame timing
    const frameMetrics = await page.evaluate(async () => {
      return new Promise<{ avgFrameTime: number; droppedFrames: number }>((resolve) => {
        const times: number[] = [];
        let lastTime = performance.now();
        let droppedFrames = 0;

        function measure() {
          const now = performance.now();
          const frameTime = now - lastTime;
          times.push(frameTime);

          if (frameTime > 33) {
            // More than 30fps threshold
            droppedFrames++;
          }

          lastTime = now;

          if (times.length < 60) {
            requestAnimationFrame(measure);
          } else {
            const avgFrameTime = times.reduce((a, b) => a + b, 0) / times.length;
            resolve({ avgFrameTime, droppedFrames });
          }
        }

        requestAnimationFrame(measure);
      });
    });

    console.log(`Average frame time: ${frameMetrics.avgFrameTime.toFixed(2)}ms`);
    console.log(`Dropped frames: ${frameMetrics.droppedFrames}/60`);

    // Allow some dropped frames, but not too many
    expect(frameMetrics.droppedFrames).toBeLessThan(10);
  });

  test("maintains responsiveness during graph interaction", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: multiAgentSession.status,
      processes: multiAgentSession.processes,
      connections: multiAgentSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    // Perform rapid interactions
    const interactionTimes: number[] = [];

    for (let i = 0; i < 10; i++) {
      const start = Date.now();
      await page.locator(".process-node").nth(i % 3).click();
      await page.waitForTimeout(50);
      interactionTimes.push(Date.now() - start);
    }

    const avgInteractionTime =
      interactionTimes.reduce((a, b) => a + b, 0) / interactionTimes.length;

    console.log(`Average interaction time: ${avgInteractionTime.toFixed(2)}ms`);
    expect(avgInteractionTime).toBeLessThan(200);
  });
});

test.describe("Performance - Memory", () => {
  test("no memory leaks during repeated operations", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: claudeCodeSession.status,
      processes: claudeCodeSession.processes,
      connections: claudeCodeSession.connections,
      signatures: mockSignatures,
    });
    await page.goto("/");
    await page.waitForTimeout(500);

    // Get initial memory
    const initialMemory = await page.evaluate(() => {
      if ((performance as any).memory) {
        return (performance as any).memory.usedJSHeapSize;
      }
      return 0;
    });

    // Perform many operations
    for (let i = 0; i < 20; i++) {
      await page.locator(".process-node").first().click();
      await page.waitForTimeout(50);
      await page.locator(".search-input").fill(`test${i}`);
      await page.waitForTimeout(50);
      await page.locator(".search-input").fill("");
    }

    // Force GC if available
    await page.evaluate(() => {
      if ((window as any).gc) {
        (window as any).gc();
      }
    });

    await page.waitForTimeout(500);

    // Get final memory
    const finalMemory = await page.evaluate(() => {
      if ((performance as any).memory) {
        return (performance as any).memory.usedJSHeapSize;
      }
      return 0;
    });

    if (initialMemory > 0) {
      const memoryGrowth = finalMemory - initialMemory;
      const growthPercent = (memoryGrowth / initialMemory) * 100;

      console.log(`Initial memory: ${(initialMemory / 1024 / 1024).toFixed(2)}MB`);
      console.log(`Final memory: ${(finalMemory / 1024 / 1024).toFixed(2)}MB`);
      console.log(`Growth: ${growthPercent.toFixed(2)}%`);

      // Allow up to 50% growth (some growth is normal)
      expect(growthPercent).toBeLessThan(50);
    }
  });
});

test.describe("Performance - Stress Tests", () => {
  test("handles 50 processes", async ({ page }) => {
    const { processes, connections } = generateLargeDataset(50, 100);

    await setupTauriMock(page, {
      connected: true,
      status: { ...claudeCodeSession.status, processesTracked: 50 },
      processes,
      connections,
      signatures: mockSignatures,
    });

    const start = Date.now();
    await page.goto("/");
    await page.waitForSelector(".process-node", { timeout: 10000 });
    const loadTime = Date.now() - start;

    const nodeCount = await page.locator(".process-node").count();
    expect(nodeCount).toBe(50);
    expect(loadTime).toBeLessThan(5000);

    console.log(`50 nodes load time: ${loadTime}ms`);
  });

  test("handles 100 processes", async ({ page }) => {
    const { processes, connections } = generateLargeDataset(100, 200);

    await setupTauriMock(page, {
      connected: true,
      status: { ...claudeCodeSession.status, processesTracked: 100 },
      processes,
      connections,
      signatures: mockSignatures,
    });

    const start = Date.now();
    await page.goto("/");
    await page.waitForSelector(".process-node", { timeout: 10000 });
    const loadTime = Date.now() - start;

    const nodeCount = await page.locator(".process-node").count();
    expect(nodeCount).toBe(100);
    expect(loadTime).toBeLessThan(8000);

    console.log(`100 nodes load time: ${loadTime}ms`);
  });

  test("handles 200 processes", async ({ page }) => {
    const { processes, connections } = generateLargeDataset(200, 400);

    await setupTauriMock(page, {
      connected: true,
      status: { ...claudeCodeSession.status, processesTracked: 200 },
      processes,
      connections,
      signatures: mockSignatures,
    });

    const start = Date.now();
    await page.goto("/");
    await page.waitForSelector(".process-node", { timeout: 15000 });
    const loadTime = Date.now() - start;

    const nodeCount = await page.locator(".process-node").count();
    expect(nodeCount).toBe(200);
    expect(loadTime).toBeLessThan(15000);

    console.log(`200 nodes load time: ${loadTime}ms`);
  });

  test("search remains fast with many processes", async ({ page }) => {
    const { processes, connections } = generateLargeDataset(100, 200);

    await setupTauriMock(page, {
      connected: true,
      status: { ...claudeCodeSession.status, processesTracked: 100 },
      processes,
      connections,
      signatures: mockSignatures,
    });

    await page.goto("/");
    await page.waitForSelector(".process-node", { timeout: 10000 });

    const searchInput = page.locator(".search-input");

    const start = Date.now();
    await searchInput.fill("process-5");
    await page.waitForTimeout(100);
    const searchTime = Date.now() - start;

    expect(searchTime).toBeLessThan(500);
    console.log(`Search with 100 nodes: ${searchTime}ms`);
  });

  test("rapid layout switching doesn't crash", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: multiAgentSession.status,
      processes: multiAgentSession.processes,
      connections: multiAgentSession.connections,
      signatures: mockSignatures,
    });

    await page.goto("/");
    await page.waitForTimeout(500);

    // Rapid layout switches
    for (let i = 0; i < 10; i++) {
      await page.locator('.control-btn:has-text("Force")').click();
      await page.waitForTimeout(50);
      await page.locator('.control-btn:has-text("Tree")').click();
      await page.waitForTimeout(50);
      await page.locator('.control-btn:has-text("Radial")').click();
      await page.waitForTimeout(50);
    }

    // App should still be functional
    const nodes = page.locator(".process-graph svg .node, .process-node");
    const count = await nodes.count();
    expect(count).toBeGreaterThan(0);
  });
});

test.describe("Performance - Metrics Collection", () => {
  test("collect performance metrics", async ({ page }) => {
    await setupTauriMock(page, {
      connected: true,
      status: claudeCodeSession.status,
      processes: claudeCodeSession.processes,
      connections: claudeCodeSession.connections,
      signatures: mockSignatures,
    });

    await page.goto("/");
    await page.waitForTimeout(2000);

    const metrics = await page.evaluate(() => {
      const timing = performance.timing;
      const paint = performance.getEntriesByType("paint");

      return {
        domContentLoaded: timing.domContentLoadedEventEnd - timing.navigationStart,
        load: timing.loadEventEnd - timing.navigationStart,
        firstPaint: paint.find((p) => p.name === "first-paint")?.startTime || 0,
        firstContentfulPaint:
          paint.find((p) => p.name === "first-contentful-paint")?.startTime || 0,
      };
    });

    console.log("\n=== Performance Metrics ===");
    console.log(`DOM Content Loaded: ${metrics.domContentLoaded}ms`);
    console.log(`Page Load: ${metrics.load}ms`);
    console.log(`First Paint: ${metrics.firstPaint}ms`);
    console.log(`First Contentful Paint: ${metrics.firstContentfulPaint}ms`);

    // Assertions
    expect(metrics.firstContentfulPaint).toBeLessThan(2000);
    expect(metrics.domContentLoaded).toBeLessThan(3000);
  });
});
