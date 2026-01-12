'use client';

import { useEffect, useState, useCallback, useRef } from 'react';
import useSWR from 'swr';
import {
  Activity,
  GitBranch,
  RefreshCw,
  Cpu,
  HardDrive,
  Clock,
  CheckCircle,
  XCircle,
  AlertCircle,
  Play,
  Square,
  Loader2,
  ZoomIn,
  ZoomOut,
  Maximize2,
} from 'lucide-react';

// Types
interface ProcessNode {
  id: string;
  label: string;
  pid: number;
  agent_type: string;
  task_id: string;
  instance_id: string;
  status: 'starting' | 'running' | 'completed' | 'failed' | 'terminated';
  is_root: boolean;
  cpu_percent: number;
  memory_mb: number;
  elapsed_secs: number;
  started_at: string;
  ended_at: string | null;
}

interface ProcessEdge {
  id: string;
  source: string;
  target: string;
}

interface ProcessStats {
  total_processes: number;
  running_processes: number;
  completed_count: number;
  failed_count: number;
  avg_cpu_percent: number;
  total_memory_bytes: number;
}

interface ProcessGraphData {
  nodes: ProcessNode[];
  edges: ProcessEdge[];
  stats: ProcessStats;
}

interface ProcessEvent {
  id: string;
  process_id: string;
  pid: number;
  task_id: string;
  instance_id: string;
  event_type: string;
  old_status: string;
  new_status: string;
  exit_code: number | null;
  message: string;
  timestamp: string;
}

interface WebSocketMessage {
  type: string;
  payload: unknown;
}

// Fetcher
const fetcher = (url: string) => fetch(url).then((res) => res.json());

// Graph layout calculation
interface LayoutNode extends ProcessNode {
  x: number;
  y: number;
  level: number;
}

function calculateLayout(nodes: ProcessNode[], edges: ProcessEdge[]): LayoutNode[] {
  if (nodes.length === 0) return [];

  // Build adjacency map
  const children = new Map<string, string[]>();
  const parents = new Map<string, string>();

  for (const edge of edges) {
    if (!children.has(edge.source)) {
      children.set(edge.source, []);
    }
    children.get(edge.source)!.push(edge.target);
    parents.set(edge.target, edge.source);
  }

  // Find root nodes (nodes without parents)
  const roots = nodes.filter((n) => !parents.has(n.id));

  // Calculate levels using BFS
  const levels = new Map<string, number>();
  const queue: { id: string; level: number }[] = roots.map((r) => ({
    id: r.id,
    level: 0,
  }));

  while (queue.length > 0) {
    const { id, level } = queue.shift()!;
    levels.set(id, level);

    const nodeChildren = children.get(id) || [];
    for (const childId of nodeChildren) {
      queue.push({ id: childId, level: level + 1 });
    }
  }

  // Handle orphan nodes
  for (const node of nodes) {
    if (!levels.has(node.id)) {
      levels.set(node.id, 0);
    }
  }

  // Group nodes by level
  const nodesByLevel = new Map<number, ProcessNode[]>();
  for (const node of nodes) {
    const level = levels.get(node.id) || 0;
    if (!nodesByLevel.has(level)) {
      nodesByLevel.set(level, []);
    }
    nodesByLevel.get(level)!.push(node);
  }

  // Calculate positions
  const nodeWidth = 220;
  const nodeHeight = 120;
  const levelGap = 150;
  const nodeGap = 40;

  const layoutNodes: LayoutNode[] = [];

  const maxLevel = Math.max(...Array.from(nodesByLevel.keys()));

  for (let level = 0; level <= maxLevel; level++) {
    const levelNodes = nodesByLevel.get(level) || [];
    const totalWidth = levelNodes.length * nodeWidth + (levelNodes.length - 1) * nodeGap;
    const startX = -totalWidth / 2 + nodeWidth / 2;

    levelNodes.forEach((node, index) => {
      layoutNodes.push({
        ...node,
        x: startX + index * (nodeWidth + nodeGap),
        y: level * levelGap,
        level,
      });
    });
  }

  return layoutNodes;
}

// Status colors
const statusColors: Record<string, { bg: string; text: string; border: string }> = {
  starting: { bg: 'bg-yellow-50', text: 'text-yellow-700', border: 'border-yellow-300' },
  running: { bg: 'bg-blue-50', text: 'text-blue-700', border: 'border-blue-300' },
  completed: { bg: 'bg-green-50', text: 'text-green-700', border: 'border-green-300' },
  failed: { bg: 'bg-red-50', text: 'text-red-700', border: 'border-red-300' },
  terminated: { bg: 'bg-gray-50', text: 'text-gray-700', border: 'border-gray-300' },
};

const statusIcons: Record<string, React.ReactNode> = {
  starting: <Loader2 className="w-4 h-4 animate-spin" />,
  running: <Play className="w-4 h-4" />,
  completed: <CheckCircle className="w-4 h-4" />,
  failed: <XCircle className="w-4 h-4" />,
  terminated: <Square className="w-4 h-4" />,
};

// Process Node Component
function ProcessNodeCard({
  node,
  selected,
  onSelect,
}: {
  node: LayoutNode;
  selected: boolean;
  onSelect: (id: string) => void;
}) {
  const colors = statusColors[node.status] || statusColors.running;

  const formatElapsed = (secs: number) => {
    if (secs < 60) return `${secs}s`;
    if (secs < 3600) return `${Math.floor(secs / 60)}m ${secs % 60}s`;
    return `${Math.floor(secs / 3600)}h ${Math.floor((secs % 3600) / 60)}m`;
  };

  return (
    <g
      transform={`translate(${node.x - 110}, ${node.y - 50})`}
      onClick={() => onSelect(node.id)}
      className="cursor-pointer"
    >
      <foreignObject width={220} height={100}>
        <div
          className={`h-full rounded-lg border-2 p-3 transition-all ${colors.bg} ${colors.border} ${
            selected ? 'ring-2 ring-primary-500 shadow-lg' : 'hover:shadow-md'
          } ${node.is_root ? 'border-solid' : 'border-dashed'}`}
        >
          {/* Header */}
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center space-x-2">
              <div className={`${colors.text}`}>{statusIcons[node.status]}</div>
              <span className="font-medium text-sm truncate max-w-[120px]">
                {node.label}
              </span>
            </div>
            <span className="text-xs bg-gray-200 px-1.5 py-0.5 rounded">
              PID {node.pid}
            </span>
          </div>

          {/* Stats */}
          <div className="grid grid-cols-3 gap-1 text-xs text-gray-600">
            <div className="flex items-center space-x-1">
              <Cpu className="w-3 h-3" />
              <span>{node.cpu_percent.toFixed(1)}%</span>
            </div>
            <div className="flex items-center space-x-1">
              <HardDrive className="w-3 h-3" />
              <span>{node.memory_mb.toFixed(0)}MB</span>
            </div>
            <div className="flex items-center space-x-1">
              <Clock className="w-3 h-3" />
              <span>{formatElapsed(node.elapsed_secs)}</span>
            </div>
          </div>

          {/* Task ID */}
          {node.task_id && (
            <div className="mt-2 text-xs text-gray-400 truncate">
              Task: {node.task_id.slice(0, 12)}...
            </div>
          )}
        </div>
      </foreignObject>
    </g>
  );
}

// Edge Component
function Edge({
  sourceNode,
  targetNode,
}: {
  sourceNode: LayoutNode;
  targetNode: LayoutNode;
}) {
  // Draw a curved line from source to target
  const startX = sourceNode.x;
  const startY = sourceNode.y + 50;
  const endX = targetNode.x;
  const endY = targetNode.y - 50;

  const midY = (startY + endY) / 2;

  const path = `M ${startX} ${startY} C ${startX} ${midY}, ${endX} ${midY}, ${endX} ${endY}`;

  return (
    <g>
      <path
        d={path}
        fill="none"
        stroke="#9CA3AF"
        strokeWidth={2}
        markerEnd="url(#arrowhead)"
      />
    </g>
  );
}

// Main Component
export function ProcessGraph({ taskId }: { taskId?: string }) {
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Build query URL
  const queryParams = new URLSearchParams();
  if (taskId) queryParams.append('task_id', taskId);

  const { data, error, mutate } = useSWR<ProcessGraphData>(
    `/api/v1/processes/graph?${queryParams.toString()}`,
    fetcher,
    { refreshInterval: 2000 }
  );

  // WebSocket connection for real-time updates
  useEffect(() => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;

    const ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      console.log('WebSocket connected');
      // Subscribe to task if specified
      if (taskId) {
        ws.send(JSON.stringify({ action: 'subscribe_task', task_id: taskId }));
      }
    };

    ws.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);

        if (message.type === 'process_event' || message.type === 'initial_graph') {
          // Refresh data on any process event
          mutate();
        }
      } catch (e) {
        console.error('Failed to parse WebSocket message:', e);
      }
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    ws.onclose = () => {
      console.log('WebSocket disconnected');
    };

    return () => {
      ws.close();
    };
  }, [taskId, mutate]);

  // Calculate layout
  const layoutNodes = data ? calculateLayout(data.nodes, data.edges) : [];

  // Build node lookup for edges
  const nodeMap = new Map(layoutNodes.map((n) => [n.id, n]));

  // Pan and zoom handlers
  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    setZoom((z) => Math.min(Math.max(z * delta, 0.25), 2));
  }, []);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button === 0) {
      setIsDragging(true);
      setDragStart({ x: e.clientX - pan.x, y: e.clientY - pan.y });
    }
  }, [pan]);

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (isDragging) {
        setPan({ x: e.clientX - dragStart.x, y: e.clientY - dragStart.y });
      }
    },
    [isDragging, dragStart]
  );

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  const resetView = useCallback(() => {
    setZoom(1);
    setPan({ x: 0, y: 0 });
  }, []);

  // Calculate viewBox to center the graph
  const minX = layoutNodes.reduce((min, n) => Math.min(min, n.x), 0) - 150;
  const maxX = layoutNodes.reduce((max, n) => Math.max(max, n.x), 0) + 150;
  const minY = -50;
  const maxY =
    layoutNodes.reduce((max, n) => Math.max(max, n.y), 0) + 100;

  const viewWidth = Math.max(maxX - minX, 500);
  const viewHeight = Math.max(maxY - minY, 300);

  if (error) {
    return (
      <div className="bg-white rounded-lg shadow-sm p-6">
        <div className="flex items-center justify-center text-red-500 space-x-2">
          <AlertCircle className="w-5 h-5" />
          <span>Failed to load process graph</span>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow-sm overflow-hidden">
      {/* Header */}
      <div className="p-4 border-b flex items-center justify-between">
        <div className="flex items-center space-x-3">
          <div className="w-8 h-8 bg-primary-100 rounded-lg flex items-center justify-center">
            <GitBranch className="w-5 h-5 text-primary-600" />
          </div>
          <div>
            <h2 className="text-lg font-semibold">Process Graph</h2>
            <p className="text-sm text-gray-500">
              {data?.stats.running_processes || 0} running, {data?.stats.total_processes || 0} total
            </p>
          </div>
        </div>

        <div className="flex items-center space-x-2">
          <button
            onClick={() => setZoom((z) => Math.min(z * 1.2, 2))}
            className="p-2 hover:bg-gray-100 rounded-lg"
            title="Zoom In"
          >
            <ZoomIn className="w-4 h-4" />
          </button>
          <button
            onClick={() => setZoom((z) => Math.max(z * 0.8, 0.25))}
            className="p-2 hover:bg-gray-100 rounded-lg"
            title="Zoom Out"
          >
            <ZoomOut className="w-4 h-4" />
          </button>
          <button
            onClick={resetView}
            className="p-2 hover:bg-gray-100 rounded-lg"
            title="Reset View"
          >
            <Maximize2 className="w-4 h-4" />
          </button>
          <button
            onClick={() => mutate()}
            className="p-2 hover:bg-gray-100 rounded-lg"
            title="Refresh"
          >
            <RefreshCw className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Stats Bar */}
      {data?.stats && (
        <div className="px-4 py-2 bg-gray-50 border-b flex items-center space-x-6 text-sm">
          <div className="flex items-center space-x-2">
            <Activity className="w-4 h-4 text-blue-500" />
            <span className="text-gray-600">Running:</span>
            <span className="font-medium">{data.stats.running_processes}</span>
          </div>
          <div className="flex items-center space-x-2">
            <CheckCircle className="w-4 h-4 text-green-500" />
            <span className="text-gray-600">Completed:</span>
            <span className="font-medium">{data.stats.completed_count}</span>
          </div>
          <div className="flex items-center space-x-2">
            <XCircle className="w-4 h-4 text-red-500" />
            <span className="text-gray-600">Failed:</span>
            <span className="font-medium">{data.stats.failed_count}</span>
          </div>
          <div className="flex items-center space-x-2">
            <Cpu className="w-4 h-4 text-gray-500" />
            <span className="text-gray-600">Avg CPU:</span>
            <span className="font-medium">{data.stats.avg_cpu_percent.toFixed(1)}%</span>
          </div>
          <div className="flex items-center space-x-2">
            <HardDrive className="w-4 h-4 text-gray-500" />
            <span className="text-gray-600">Memory:</span>
            <span className="font-medium">
              {(data.stats.total_memory_bytes / (1024 * 1024)).toFixed(0)} MB
            </span>
          </div>
        </div>
      )}

      {/* Graph Canvas */}
      <div
        ref={containerRef}
        className="relative overflow-hidden bg-gray-50"
        style={{ height: '500px' }}
        onWheel={handleWheel}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      >
        {!data || layoutNodes.length === 0 ? (
          <div className="flex items-center justify-center h-full text-gray-500">
            <div className="text-center">
              <Activity className="w-12 h-12 mx-auto mb-3 text-gray-300" />
              <p>No processes to display</p>
              <p className="text-sm">Start a task to see processes here</p>
            </div>
          </div>
        ) : (
          <svg
            ref={svgRef}
            width="100%"
            height="100%"
            viewBox={`${minX} ${minY} ${viewWidth} ${viewHeight}`}
            className={isDragging ? 'cursor-grabbing' : 'cursor-grab'}
            style={{
              transform: `scale(${zoom}) translate(${pan.x / zoom}px, ${pan.y / zoom}px)`,
              transformOrigin: 'center',
            }}
          >
            {/* Arrow marker definition */}
            <defs>
              <marker
                id="arrowhead"
                markerWidth="10"
                markerHeight="7"
                refX="9"
                refY="3.5"
                orient="auto"
              >
                <polygon points="0 0, 10 3.5, 0 7" fill="#9CA3AF" />
              </marker>
            </defs>

            {/* Edges */}
            {data.edges.map((edge) => {
              const source = nodeMap.get(edge.source);
              const target = nodeMap.get(edge.target);
              if (!source || !target) return null;
              return <Edge key={edge.id} sourceNode={source} targetNode={target} />;
            })}

            {/* Nodes */}
            {layoutNodes.map((node) => (
              <ProcessNodeCard
                key={node.id}
                node={node}
                selected={selectedNode === node.id}
                onSelect={setSelectedNode}
              />
            ))}
          </svg>
        )}
      </div>

      {/* Selected Node Details */}
      {selectedNode && (
        <NodeDetails
          node={layoutNodes.find((n) => n.id === selectedNode)}
          onClose={() => setSelectedNode(null)}
        />
      )}
    </div>
  );
}

// Node Details Panel
function NodeDetails({
  node,
  onClose,
}: {
  node?: LayoutNode;
  onClose: () => void;
}) {
  if (!node) return null;

  const colors = statusColors[node.status] || statusColors.running;

  return (
    <div className="border-t p-4 bg-gray-50">
      <div className="flex items-center justify-between mb-3">
        <h3 className="font-medium">Process Details</h3>
        <button
          onClick={onClose}
          className="text-gray-400 hover:text-gray-600"
        >
          &times;
        </button>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
        <div>
          <span className="text-gray-500">PID</span>
          <p className="font-medium">{node.pid}</p>
        </div>
        <div>
          <span className="text-gray-500">Status</span>
          <p className={`font-medium ${colors.text}`}>{node.status}</p>
        </div>
        <div>
          <span className="text-gray-500">Agent Type</span>
          <p className="font-medium">{node.agent_type || '-'}</p>
        </div>
        <div>
          <span className="text-gray-500">Root Process</span>
          <p className="font-medium">{node.is_root ? 'Yes' : 'No'}</p>
        </div>
        <div>
          <span className="text-gray-500">Task ID</span>
          <p className="font-medium truncate">{node.task_id || '-'}</p>
        </div>
        <div>
          <span className="text-gray-500">Instance ID</span>
          <p className="font-medium truncate">{node.instance_id || '-'}</p>
        </div>
        <div>
          <span className="text-gray-500">CPU Usage</span>
          <p className="font-medium">{node.cpu_percent.toFixed(1)}%</p>
        </div>
        <div>
          <span className="text-gray-500">Memory</span>
          <p className="font-medium">{node.memory_mb.toFixed(1)} MB</p>
        </div>
        <div className="md:col-span-2">
          <span className="text-gray-500">Started At</span>
          <p className="font-medium">
            {new Date(node.started_at).toLocaleString()}
          </p>
        </div>
        {node.ended_at && (
          <div className="md:col-span-2">
            <span className="text-gray-500">Ended At</span>
            <p className="font-medium">
              {new Date(node.ended_at).toLocaleString()}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

export default ProcessGraph;
