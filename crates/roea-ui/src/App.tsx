import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Header } from "./components/Header";
import { Sidebar } from "./components/Sidebar";
import { ProcessGraph } from "./components/ProcessGraph";
import { DetailsPanel } from "./components/DetailsPanel";
import { StatsBar } from "./components/StatsBar";
import { Process, AgentSignature, AgentStatus } from "./lib/types";

function App() {
  const [connected, setConnected] = useState(false);
  const [status, setStatus] = useState<AgentStatus | null>(null);
  const [processes, setProcesses] = useState<Process[]>([]);
  const [signatures, setSignatures] = useState<AgentSignature[]>([]);
  const [selectedProcess, setSelectedProcess] = useState<Process | null>(null);
  const [selectedAgentType, setSelectedAgentType] = useState<string | null>(null);

  // Connect to agent on mount
  useEffect(() => {
    connectToAgent();
  }, []);

  // Refresh data periodically when connected
  useEffect(() => {
    if (!connected) return;

    const refresh = () => {
      refreshStatus();
      refreshProcesses();
    };

    refresh();
    const interval = setInterval(refresh, 2000);
    return () => clearInterval(interval);
  }, [connected]);

  const connectToAgent = async () => {
    try {
      const result = await invoke<boolean>("connect_to_agent", {});
      setConnected(result);
      if (result) {
        await refreshSignatures();
      }
    } catch (e) {
      console.error("Failed to connect:", e);
      setConnected(false);
    }
  };

  const refreshStatus = async () => {
    try {
      const result = await invoke<AgentStatus>("get_status", {});
      setStatus(result);
    } catch (e) {
      console.error("Failed to get status:", e);
    }
  };

  const refreshProcesses = async () => {
    try {
      const result = await invoke<Process[]>("get_processes", {});
      setProcesses(result);
    } catch (e) {
      console.error("Failed to get processes:", e);
    }
  };

  const refreshSignatures = async () => {
    try {
      const result = await invoke<AgentSignature[]>("get_signatures", {});
      setSignatures(result);
    } catch (e) {
      console.error("Failed to get signatures:", e);
    }
  };

  // Filter processes by agent type
  const filteredProcesses = selectedAgentType
    ? processes.filter((p) => p.agentType === selectedAgentType)
    : processes;

  // Get agent-specific processes grouped
  const agentProcesses = signatures.map((sig) => ({
    signature: sig,
    count: processes.filter((p) => p.agentType === sig.name).length,
  }));

  return (
    <div className="app">
      <Header
        connected={connected}
        status={status}
        onReconnect={connectToAgent}
      />
      <div className="main-content">
        <Sidebar
          agents={agentProcesses}
          selectedAgent={selectedAgentType}
          onSelectAgent={setSelectedAgentType}
        />
        <ProcessGraph
          processes={filteredProcesses}
          selectedProcess={selectedProcess}
          onSelectProcess={setSelectedProcess}
        />
        {selectedProcess && (
          <DetailsPanel
            process={selectedProcess}
            onClose={() => setSelectedProcess(null)}
          />
        )}
      </div>
      <StatsBar
        processCount={processes.length}
        agentCount={agentProcesses.filter((a) => a.count > 0).length}
        eventsCollected={status?.eventsCollected ?? 0}
      />
    </div>
  );
}

export default App;
