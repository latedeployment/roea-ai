// Type definitions for roea-ai UI

export interface Process {
  id: string;
  pid: number;
  ppid: number;
  name: string;
  cmdline: string;
  exePath: string;
  agentType: string;
  startTime: number;
  endTime: number;
  user: string;
  cwd: string;
}

export interface Connection {
  id: string;
  processId: string;
  pid: number;
  protocol: string;
  localAddr: string;
  localPort: number;
  remoteAddr: string;
  remotePort: number;
  state: string;
  timestamp: number;
}

export interface FileOp {
  id: string;
  processId: string;
  pid: number;
  operation: string;
  path: string;
  newPath: string;
  timestamp: number;
}

export interface AgentSignature {
  name: string;
  displayName: string;
  icon: string;
  expectedEndpoints: string[];
  childProcessTracking: boolean;
}

export interface AgentStatus {
  running: boolean;
  platform: string;
  elevatedPrivileges: boolean;
  uptimeSeconds: number;
  processesTracked: number;
  eventsCollected: number;
}

export interface AgentWithCount {
  signature: AgentSignature;
  count: number;
}

// Graph node for D3 visualization
export interface ProcessNode extends d3.SimulationNodeDatum {
  id: string;
  pid: number;
  ppid: number;
  name: string;
  agentType: string;
  isAgent: boolean;
  isActive?: boolean;
  connectionCount?: number;
}

export interface ProcessLink extends d3.SimulationLinkDatum<ProcessNode> {
  source: ProcessNode | string;
  target: ProcessNode | string;
  type?: "parent-child" | "network";
}
