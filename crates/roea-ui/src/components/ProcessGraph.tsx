import { useEffect, useRef, useMemo, useState, useCallback } from "react";
import * as d3 from "d3";
import { Process, ProcessNode, ProcessLink, Connection } from "../lib/types";

interface ProcessGraphProps {
  processes: Process[];
  connections?: Connection[];
  selectedProcess: Process | null;
  onSelectProcess: (process: Process | null) => void;
}

type LayoutType = "force" | "tree" | "radial";

const agentColors: Record<string, string> = {
  claude_code: "#d97706",
  cursor: "#8b5cf6",
  aider: "#10b981",
  windsurf: "#06b6d4",
  copilot: "#6366f1",
  continue_dev: "#ec4899",
};

const defaultNodeColor = "#4b5563";
const activeColor = "#22c55e";
const exitedColor = "#6b7280";

export function ProcessGraph({
  processes,
  connections = [],
  selectedProcess,
  onSelectProcess,
}: ProcessGraphProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const simulationRef = useRef<d3.Simulation<ProcessNode, ProcessLink> | null>(null);
  const [layoutType, setLayoutType] = useState<LayoutType>("force");
  const [showNetworkOverlay, setShowNetworkOverlay] = useState(false);
  const [zoomLevel, setZoomLevel] = useState(1);

  // Build graph data from processes with memoization
  const { nodes, links, networkLinks } = useMemo(() => {
    const nodeMap = new Map<number, ProcessNode>();
    const processIdMap = new Map<string, ProcessNode>();
    const nodes: ProcessNode[] = [];
    const links: ProcessLink[] = [];
    const networkLinks: ProcessLink[] = [];

    // Create nodes for each process
    processes.forEach((p) => {
      const isActive = p.endTime === 0;
      const node: ProcessNode = {
        id: p.id,
        pid: p.pid,
        ppid: p.ppid,
        name: p.name,
        agentType: p.agentType,
        isAgent: !!p.agentType,
        isActive,
        connectionCount: 0,
      };
      nodes.push(node);
      nodeMap.set(p.pid, node);
      processIdMap.set(p.id, node);
    });

    // Count connections per process
    connections.forEach((conn) => {
      const node = processIdMap.get(conn.processId);
      if (node) {
        node.connectionCount = (node.connectionCount || 0) + 1;
      }
    });

    // Create links from parent-child relationships
    processes.forEach((p) => {
      if (p.ppid && nodeMap.has(p.ppid)) {
        links.push({
          source: nodeMap.get(p.ppid)!.id,
          target: nodeMap.get(p.pid)!.id,
          type: "parent-child",
        });
      }
    });

    // Create network connection links between processes
    if (showNetworkOverlay) {
      const processConnections = new Map<string, Set<string>>();

      connections.forEach((conn) => {
        // Find processes that share the same remote endpoint
        const key = `${conn.remoteAddr}:${conn.remotePort}`;
        if (!processConnections.has(key)) {
          processConnections.set(key, new Set());
        }
        processConnections.get(key)!.add(conn.processId);
      });

      // Create links between processes sharing endpoints
      processConnections.forEach((processIds) => {
        const ids = Array.from(processIds);
        for (let i = 0; i < ids.length; i++) {
          for (let j = i + 1; j < ids.length; j++) {
            if (processIdMap.has(ids[i]) && processIdMap.has(ids[j])) {
              networkLinks.push({
                source: ids[i],
                target: ids[j],
                type: "network",
              });
            }
          }
        }
      });
    }

    return { nodes, links, networkLinks };
  }, [processes, connections, showNetworkOverlay]);

  // Build tree hierarchy for tree layout
  const buildHierarchy = useCallback(() => {
    if (nodes.length === 0) return null;

    const nodeById = new Map(nodes.map(n => [n.id, { ...n, children: [] as typeof nodes }]));
    const pidToId = new Map(nodes.map(n => [n.pid, n.id]));
    let roots: typeof nodes = [];

    nodes.forEach(node => {
      const parentId = node.ppid ? pidToId.get(node.ppid) : null;
      if (parentId && nodeById.has(parentId)) {
        nodeById.get(parentId)!.children.push(nodeById.get(node.id)!);
      } else {
        roots.push(nodeById.get(node.id)!);
      }
    });

    // If multiple roots, create a virtual root
    if (roots.length > 1) {
      return {
        id: "root",
        name: "System",
        children: roots,
        isAgent: false,
        isActive: true,
        virtual: true,
      };
    }
    return roots[0] || null;
  }, [nodes]);

  // Main render effect
  useEffect(() => {
    if (!svgRef.current || !containerRef.current || nodes.length === 0) return;

    const svg = d3.select(svgRef.current);
    const container = containerRef.current;
    const width = container.clientWidth;
    const height = container.clientHeight;

    // Clear previous content with transition
    svg.selectAll("*").remove();

    // Create defs for gradients and markers
    const defs = svg.append("defs");

    // Arrow marker for links
    defs.append("marker")
      .attr("id", "arrowhead")
      .attr("viewBox", "0 -5 10 10")
      .attr("refX", 20)
      .attr("refY", 0)
      .attr("markerWidth", 6)
      .attr("markerHeight", 6)
      .attr("orient", "auto")
      .append("path")
      .attr("d", "M0,-5L10,0L0,5")
      .attr("fill", "#555");

    // Network link marker (different style)
    defs.append("marker")
      .attr("id", "network-arrow")
      .attr("viewBox", "0 -5 10 10")
      .attr("refX", 20)
      .attr("refY", 0)
      .attr("markerWidth", 4)
      .attr("markerHeight", 4)
      .attr("orient", "auto")
      .append("path")
      .attr("d", "M0,-5L10,0L0,5")
      .attr("fill", "#3b82f6");

    // Create zoom behavior
    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on("zoom", (event) => {
        g.attr("transform", event.transform);
        setZoomLevel(event.transform.k);
      });

    svg.call(zoom);

    // Create main group for zoom/pan
    const g = svg.append("g");

    // Layout-specific rendering
    if (layoutType === "tree" || layoutType === "radial") {
      renderTreeLayout(g, width, height);
    } else {
      renderForceLayout(g, width, height);
    }

    function renderForceLayout(g: d3.Selection<SVGGElement, unknown, null, undefined>, width: number, height: number) {
      // Stop previous simulation
      if (simulationRef.current) {
        simulationRef.current.stop();
      }

      // Create simulation with optimized parameters for performance
      const simulation = d3
        .forceSimulation<ProcessNode>(nodes)
        .force(
          "link",
          d3.forceLink<ProcessNode, ProcessLink>(links)
            .id((d) => d.id)
            .distance(80)
            .strength(0.5)
        )
        .force("charge", d3.forceManyBody()
          .strength((d) => (d as ProcessNode).isAgent ? -300 : -150)
          .distanceMax(300)
        )
        .force("center", d3.forceCenter(width / 2, height / 2))
        .force("collision", d3.forceCollide().radius((d) => (d as ProcessNode).isAgent ? 25 : 18))
        .force("x", d3.forceX(width / 2).strength(0.05))
        .force("y", d3.forceY(height / 2).strength(0.05))
        .alphaDecay(0.02)
        .velocityDecay(0.4);

      simulationRef.current = simulation;

      // Draw network links (if overlay enabled)
      if (showNetworkOverlay && networkLinks.length > 0) {
        g.append("g")
          .attr("class", "network-links")
          .selectAll("line")
          .data(networkLinks)
          .join("line")
          .attr("stroke", "#3b82f6")
          .attr("stroke-width", 1.5)
          .attr("stroke-opacity", 0.4)
          .attr("stroke-dasharray", "4,4");
      }

      // Draw parent-child links
      const link = g
        .append("g")
        .attr("class", "links")
        .selectAll("line")
        .data(links)
        .join("line")
        .attr("stroke", "#444")
        .attr("stroke-width", 1.5)
        .attr("stroke-opacity", 0.6)
        .attr("marker-end", "url(#arrowhead)");

      // Draw nodes
      const node = g
        .append("g")
        .attr("class", "nodes")
        .selectAll<SVGGElement, ProcessNode>("g")
        .data(nodes)
        .join("g")
        .attr("class", "process-node")
        .style("cursor", "pointer")
        .on("click", (event, d) => {
          event.stopPropagation();
          const process = processes.find((p) => p.id === d.id);
          if (process) {
            onSelectProcess(selectedProcess?.id === process.id ? null : process);
          }
        });

      // Add outer ring for agents
      node.filter(d => d.isAgent)
        .append("circle")
        .attr("r", 18)
        .attr("fill", "none")
        .attr("stroke", (d) => agentColors[d.agentType] || defaultNodeColor)
        .attr("stroke-width", 2)
        .attr("stroke-opacity", 0.3);

      // Add main circle
      node
        .append("circle")
        .attr("r", (d) => d.isAgent ? 14 : 8)
        .attr("fill", (d) => {
          if (d.isAgent) {
            return agentColors[d.agentType] || defaultNodeColor;
          }
          return d.isActive ? "#4b5563" : "#374151";
        })
        .attr("stroke", (d) => {
          if (selectedProcess?.id === d.id) return "#fff";
          return d.isActive ? activeColor : exitedColor;
        })
        .attr("stroke-width", (d) => selectedProcess?.id === d.id ? 3 : 2);

      // Add status indicator dot
      node
        .append("circle")
        .attr("r", 3)
        .attr("cx", (d) => d.isAgent ? 10 : 6)
        .attr("cy", (d) => d.isAgent ? -10 : -6)
        .attr("fill", (d) => d.isActive ? activeColor : exitedColor);

      // Add network activity indicator (small bars)
      node.filter(d => (d.connectionCount || 0) > 0)
        .append("g")
        .attr("class", "network-indicator")
        .attr("transform", (d) => `translate(${d.isAgent ? -10 : -6}, ${d.isAgent ? 10 : 6})`)
        .each(function(d) {
          const g = d3.select(this);
          const count = Math.min(d.connectionCount || 0, 5);
          for (let i = 0; i < count; i++) {
            g.append("rect")
              .attr("x", i * 3)
              .attr("y", 0)
              .attr("width", 2)
              .attr("height", 4 + Math.random() * 4)
              .attr("fill", "#3b82f6")
              .attr("opacity", 0.7);
          }
        });

      // Add labels
      node
        .append("text")
        .text((d) => d.name.length > 12 ? d.name.substring(0, 10) + "..." : d.name)
        .attr("x", 0)
        .attr("y", (d) => d.isAgent ? 28 : 20)
        .attr("text-anchor", "middle")
        .attr("font-size", "10px")
        .attr("fill", "#9ca3af")
        .attr("font-weight", (d) => d.isAgent ? "500" : "400");

      // Add PID label on hover
      node
        .append("text")
        .attr("class", "pid-label")
        .text((d) => `PID: ${d.pid}`)
        .attr("x", 0)
        .attr("y", (d) => d.isAgent ? 40 : 32)
        .attr("text-anchor", "middle")
        .attr("font-size", "8px")
        .attr("fill", "#6b7280")
        .attr("opacity", 0);

      // Show PID on hover
      node
        .on("mouseenter", function() {
          d3.select(this).select(".pid-label").attr("opacity", 1);
        })
        .on("mouseleave", function() {
          d3.select(this).select(".pid-label").attr("opacity", 0);
        });

      // Drag behavior
      const drag = d3
        .drag<SVGGElement, ProcessNode>()
        .on("start", (event, d) => {
          if (!event.active) simulation.alphaTarget(0.3).restart();
          d.fx = d.x;
          d.fy = d.y;
        })
        .on("drag", (event, d) => {
          d.fx = event.x;
          d.fy = event.y;
        })
        .on("end", (event, d) => {
          if (!event.active) simulation.alphaTarget(0);
          d.fx = null;
          d.fy = null;
        });

      node.call(drag);

      // Update positions on tick with throttling for performance
      let lastTick = 0;
      simulation.on("tick", () => {
        const now = Date.now();
        if (now - lastTick < 16) return; // ~60fps cap
        lastTick = now;

        link
          .attr("x1", (d) => (d.source as ProcessNode).x!)
          .attr("y1", (d) => (d.source as ProcessNode).y!)
          .attr("x2", (d) => (d.target as ProcessNode).x!)
          .attr("y2", (d) => (d.target as ProcessNode).y!);

        if (showNetworkOverlay) {
          g.selectAll(".network-links line")
            .attr("x1", (d: any) => {
              const source = nodes.find(n => n.id === d.source);
              return source?.x || 0;
            })
            .attr("y1", (d: any) => {
              const source = nodes.find(n => n.id === d.source);
              return source?.y || 0;
            })
            .attr("x2", (d: any) => {
              const target = nodes.find(n => n.id === d.target);
              return target?.x || 0;
            })
            .attr("y2", (d: any) => {
              const target = nodes.find(n => n.id === d.target);
              return target?.y || 0;
            });
        }

        node.attr("transform", (d) => `translate(${d.x},${d.y})`);
      });
    }

    function renderTreeLayout(g: d3.Selection<SVGGElement, unknown, null, undefined>, width: number, height: number) {
      const hierarchy = buildHierarchy();
      if (!hierarchy) return;

      const root = d3.hierarchy(hierarchy);

      let layout;
      if (layoutType === "radial") {
        layout = d3.tree<typeof hierarchy>()
          .size([2 * Math.PI, Math.min(width, height) / 2 - 100])
          .separation((a, b) => (a.parent === b.parent ? 1 : 2) / a.depth);
      } else {
        layout = d3.tree<typeof hierarchy>()
          .size([width - 100, height - 100]);
      }

      layout(root);

      // Transform to center
      g.attr("transform", `translate(${width / 2}, ${height / 2})`);

      // Draw links
      if (layoutType === "radial") {
        g.selectAll(".link")
          .data(root.links())
          .join("path")
          .attr("class", "link")
          .attr("fill", "none")
          .attr("stroke", "#444")
          .attr("stroke-width", 1.5)
          .attr("d", d3.linkRadial<any, any>()
            .angle((d: any) => d.x)
            .radius((d: any) => d.y));
      } else {
        g.attr("transform", `translate(50, 50)`);
        g.selectAll(".link")
          .data(root.links())
          .join("path")
          .attr("class", "link")
          .attr("fill", "none")
          .attr("stroke", "#444")
          .attr("stroke-width", 1.5)
          .attr("d", d3.linkVertical<any, any>()
            .x((d: any) => d.x)
            .y((d: any) => d.y));
      }

      // Draw nodes
      const node = g.selectAll(".node")
        .data(root.descendants())
        .join("g")
        .attr("class", "node")
        .attr("transform", (d: any) => {
          if (layoutType === "radial") {
            return `rotate(${d.x * 180 / Math.PI - 90}) translate(${d.y}, 0)`;
          }
          return `translate(${d.x}, ${d.y})`;
        })
        .style("cursor", "pointer")
        .on("click", (_event, d: any) => {
          if (d.data.virtual) return;
          const process = processes.find((p) => p.id === d.data.id);
          if (process) {
            onSelectProcess(selectedProcess?.id === process.id ? null : process);
          }
        });

      node.append("circle")
        .attr("r", (d: any) => d.data.isAgent ? 12 : 8)
        .attr("fill", (d: any) => {
          if (d.data.virtual) return "#374151";
          if (d.data.isAgent) return agentColors[d.data.agentType] || defaultNodeColor;
          return d.data.isActive ? "#4b5563" : "#374151";
        })
        .attr("stroke", (d: any) => {
          if (selectedProcess?.id === d.data.id) return "#fff";
          return d.data.isActive ? activeColor : exitedColor;
        })
        .attr("stroke-width", (d: any) => selectedProcess?.id === d.data.id ? 3 : 2);

      node.append("text")
        .attr("dy", "0.31em")
        .attr("x", (d: any) => layoutType === "radial" ? (d.x < Math.PI ? 16 : -16) : 0)
        .attr("y", () => layoutType === "radial" ? 0 : 20)
        .attr("text-anchor", (d: any) => {
          if (layoutType === "radial") return d.x < Math.PI ? "start" : "end";
          return "middle";
        })
        .attr("transform", (d: any) => layoutType === "radial" && d.x >= Math.PI ? "rotate(180)" : null)
        .text((d: any) => d.data.name?.substring(0, 12) || "")
        .attr("font-size", "9px")
        .attr("fill", "#9ca3af");
    }

    // Click on background to deselect
    svg.on("click", () => {
      onSelectProcess(null);
    });

    // Cleanup
    return () => {
      if (simulationRef.current) {
        simulationRef.current.stop();
      }
    };
  }, [nodes, links, networkLinks, processes, selectedProcess, onSelectProcess, layoutType, showNetworkOverlay, buildHierarchy]);

  // Empty state
  if (processes.length === 0) {
    return (
      <div className="graph-container">
        <div className="empty-state">
          <div className="empty-state-icon">🔍</div>
          <div className="empty-state-title">No Processes Found</div>
          <div className="empty-state-description">
            Start an AI coding agent to see it appear in the process tree.
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="graph-container" ref={containerRef}>
      <div className="graph-controls">
        <div className="control-group">
          <span className="control-label">Layout:</span>
          <button
            className={`control-btn ${layoutType === "force" ? "active" : ""}`}
            onClick={() => setLayoutType("force")}
            title="Force-directed layout"
          >
            Force
          </button>
          <button
            className={`control-btn ${layoutType === "tree" ? "active" : ""}`}
            onClick={() => setLayoutType("tree")}
            title="Tree layout"
          >
            Tree
          </button>
          <button
            className={`control-btn ${layoutType === "radial" ? "active" : ""}`}
            onClick={() => setLayoutType("radial")}
            title="Radial tree layout"
          >
            Radial
          </button>
        </div>
        <div className="control-group">
          <label className="control-checkbox">
            <input
              type="checkbox"
              checked={showNetworkOverlay}
              onChange={(e) => setShowNetworkOverlay(e.target.checked)}
            />
            <span>Network Overlay</span>
          </label>
        </div>
        <div className="control-group">
          <span className="zoom-indicator">Zoom: {Math.round(zoomLevel * 100)}%</span>
        </div>
      </div>
      <svg ref={svgRef} className="graph-canvas" />
      <div className="graph-legend">
        <div className="legend-item">
          <span className="legend-dot active"></span>
          <span>Active</span>
        </div>
        <div className="legend-item">
          <span className="legend-dot exited"></span>
          <span>Exited</span>
        </div>
        <div className="legend-item">
          <span className="legend-dot agent" style={{ background: "#d97706" }}></span>
          <span>Claude</span>
        </div>
        <div className="legend-item">
          <span className="legend-dot agent" style={{ background: "#8b5cf6" }}></span>
          <span>Cursor</span>
        </div>
      </div>
    </div>
  );
}
