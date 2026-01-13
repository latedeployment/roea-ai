import { useEffect, useRef, useMemo } from "react";
import * as d3 from "d3";
import { Process, ProcessNode, ProcessLink } from "../lib/types";

interface ProcessGraphProps {
  processes: Process[];
  selectedProcess: Process | null;
  onSelectProcess: (process: Process | null) => void;
}

const agentColors: Record<string, string> = {
  claude_code: "#d97706",
  cursor: "#8b5cf6",
  aider: "#10b981",
  windsurf: "#06b6d4",
  copilot: "#6366f1",
  continue_dev: "#6b7280",
};

export function ProcessGraph({
  processes,
  selectedProcess,
  onSelectProcess,
}: ProcessGraphProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Build graph data from processes
  const { nodes, links } = useMemo(() => {
    const nodeMap = new Map<number, ProcessNode>();
    const nodes: ProcessNode[] = [];
    const links: ProcessLink[] = [];

    // Create nodes for each process
    processes.forEach((p) => {
      const node: ProcessNode = {
        id: p.id,
        pid: p.pid,
        ppid: p.ppid,
        name: p.name,
        agentType: p.agentType,
        isAgent: !!p.agentType,
      };
      nodes.push(node);
      nodeMap.set(p.pid, node);
    });

    // Create links from parent-child relationships
    processes.forEach((p) => {
      if (p.ppid && nodeMap.has(p.ppid)) {
        links.push({
          source: nodeMap.get(p.ppid)!.id,
          target: nodeMap.get(p.pid)!.id,
        });
      }
    });

    return { nodes, links };
  }, [processes]);

  useEffect(() => {
    if (!svgRef.current || !containerRef.current || nodes.length === 0) return;

    const svg = d3.select(svgRef.current);
    const container = containerRef.current;
    const width = container.clientWidth;
    const height = container.clientHeight;

    // Clear previous content
    svg.selectAll("*").remove();

    // Create zoom behavior
    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on("zoom", (event) => {
        g.attr("transform", event.transform);
      });

    svg.call(zoom);

    // Create main group for zoom/pan
    const g = svg.append("g");

    // Create simulation
    const simulation = d3
      .forceSimulation<ProcessNode>(nodes)
      .force(
        "link",
        d3
          .forceLink<ProcessNode, ProcessLink>(links)
          .id((d) => d.id)
          .distance(60)
      )
      .force("charge", d3.forceManyBody().strength(-200))
      .force("center", d3.forceCenter(width / 2, height / 2))
      .force("collision", d3.forceCollide().radius(30));

    // Draw links
    const link = g
      .append("g")
      .attr("class", "links")
      .selectAll("line")
      .data(links)
      .join("line")
      .attr("stroke", "#333")
      .attr("stroke-width", 1)
      .attr("stroke-opacity", 0.6);

    // Draw nodes
    const node = g
      .append("g")
      .attr("class", "nodes")
      .selectAll<SVGGElement, ProcessNode>("g")
      .data(nodes)
      .join("g")
      .attr("class", "process-node")
      .style("cursor", "pointer")
      .on("click", (_, d) => {
        const process = processes.find((p) => p.id === d.id);
        if (process) {
          onSelectProcess(
            selectedProcess?.id === process.id ? null : process
          );
        }
      });

    // Add circles to nodes
    node
      .append("circle")
      .attr("r", (d) => (d.isAgent ? 12 : 8))
      .attr("fill", (d) =>
        d.isAgent
          ? agentColors[d.agentType] || "#6b7280"
          : "#4b5563"
      )
      .attr("stroke", (d) =>
        selectedProcess?.id === d.id ? "#fff" : "#1f2937"
      )
      .attr("stroke-width", (d) =>
        selectedProcess?.id === d.id ? 3 : 2
      );

    // Add labels to nodes
    node
      .append("text")
      .text((d) => d.name.substring(0, 12))
      .attr("x", 0)
      .attr("y", (d) => (d.isAgent ? 24 : 20))
      .attr("text-anchor", "middle")
      .attr("font-size", "9px")
      .attr("fill", "#9ca3af");

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

    // Update positions on tick
    simulation.on("tick", () => {
      link
        .attr("x1", (d) => (d.source as ProcessNode).x!)
        .attr("y1", (d) => (d.source as ProcessNode).y!)
        .attr("x2", (d) => (d.target as ProcessNode).x!)
        .attr("y2", (d) => (d.target as ProcessNode).y!);

      node.attr("transform", (d) => `translate(${d.x},${d.y})`);
    });

    // Cleanup
    return () => {
      simulation.stop();
    };
  }, [nodes, links, processes, selectedProcess, onSelectProcess]);

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
      <svg ref={svgRef} className="graph-canvas" />
    </div>
  );
}
