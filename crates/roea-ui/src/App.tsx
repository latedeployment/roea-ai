import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Header } from "./components/Header";
import { Sidebar } from "./components/Sidebar";
import { SearchBar } from "./components/SearchBar";
import { ProcessGraph } from "./components/ProcessGraph";
import { DetailsPanel } from "./components/DetailsPanel";
import { StatsBar } from "./components/StatsBar";
import { Process, Connection, AgentSignature, AgentStatus } from "./lib/types";

// Helper to trigger file download in browser
function downloadFile(content: string, filename: string, mimeType: string) {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

function App() {
  const [connected, setConnected] = useState(false);
  const [status, setStatus] = useState<AgentStatus | null>(null);
  const [processes, setProcesses] = useState<Process[]>([]);
  const [connections, setConnections] = useState<Connection[]>([]);
  const [filteredProcesses, setFilteredProcesses] = useState<Process[]>([]);
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
      refreshConnections();
    };

    refresh();
    const interval = setInterval(refresh, 2000);
    return () => clearInterval(interval);
  }, [connected]);

  const connectToAgent = async () => {
    try {
      console.log("Attempting to connect to agent at http://127.0.0.1:50051...");
      const result = await invoke<boolean>("connect_to_agent", {});
      console.log("Connection result:", result);
      setConnected(result);
      if (result) {
        await refreshSignatures();
        // Verify connection works by fetching status
        await refreshStatus();
      }
    } catch (e) {
      console.error("Failed to connect to agent:", e);
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

  const refreshConnections = async () => {
    try {
      const result = await invoke<Connection[]>("get_connections", {});
      setConnections(result);
    } catch (e) {
      console.error("Failed to get connections:", e);
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

  // Get processes filtered by sidebar agent type selection
  const agentFilteredProcesses = selectedAgentType
    ? processes.filter((p) => p.agentType === selectedAgentType)
    : processes;

  // Get agent-specific processes grouped
  const agentProcesses = signatures.map((sig) => ({
    signature: sig,
    count: processes.filter((p) => p.agentType === sig.name).length,
  }));

  // Handle search bar filter updates
  const handleFilteredProcesses = useCallback((filtered: Process[]) => {
    setFilteredProcesses(filtered);
  }, []);

  // Export functionality
  const handleExport = useCallback((format: "json" | "csv") => {
    const dataToExport = filteredProcesses.length > 0 ? filteredProcesses : agentFilteredProcesses;

    if (format === "json") {
      const json = JSON.stringify(dataToExport, null, 2);
      downloadFile(json, "processes.json", "application/json");
    } else {
      const headers = ["pid", "name", "agentType", "startTime", "endTime", "parentPid", "exePath", "cmdline"];
      const csvRows = [
        headers.join(","),
        ...dataToExport.map((p) =>
          headers.map((h) => {
            const value = p[h as keyof Process];
            if (value === undefined || value === null) return "";
            const str = String(value);
            return str.includes(",") || str.includes('"') || str.includes("\n")
              ? `"${str.replace(/"/g, '""')}"`
              : str;
          }).join(",")
        ),
      ];
      downloadFile(csvRows.join("\n"), "processes.csv", "text/csv");
    }
  }, [filteredProcesses, agentFilteredProcesses]);

  // Display processes (search filter takes precedence)
  const displayProcesses = filteredProcesses.length > 0 || processes.length === 0
    ? filteredProcesses
    : agentFilteredProcesses;

  return (
    <div className="app">
      <Header
        connected={connected}
        status={status}
        onReconnect={connectToAgent}
      />
      <SearchBar
        processes={agentFilteredProcesses}
        onFilteredProcesses={handleFilteredProcesses}
        onExport={handleExport}
      />
      <div className="main-content">
        <Sidebar
          agents={agentProcesses}
          selectedAgent={selectedAgentType}
          onSelectAgent={setSelectedAgentType}
        />
        <ProcessGraph
          processes={displayProcesses}
          connections={connections}
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
