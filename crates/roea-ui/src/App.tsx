import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Header } from "./components/Header";
import { SearchBar } from "./components/SearchBar";
import { ProcessTree } from "./components/ProcessTree";
import { EventTable } from "./components/EventTable";
import { DetailsPanel } from "./components/DetailsPanel";
import { StatsBar } from "./components/StatsBar";
import { Process, Connection, FileOp, AgentSignature, AgentStatus } from "./lib/types";

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
  const [fileOps, setFileOps] = useState<FileOp[]>([]);
  const [filteredProcesses, setFilteredProcesses] = useState<Process[]>([]);
  const [signatures, setSignatures] = useState<AgentSignature[]>([]);
  const [selectedProcess, setSelectedProcess] = useState<Process | null>(null);

  // Connect to agent on mount
  useEffect(() => {
    connectToAgent();
  }, []);

  // Refresh data periodically when connected
  // Also check connection health via ping
  useEffect(() => {
    if (!connected) return;

    const refresh = async () => {
      // First, check if agent is still alive
      try {
        const isAlive = await invoke<boolean>("ping", {});
        if (!isAlive) {
          console.warn("Agent ping failed - marking as disconnected");
          setConnected(false);
          setStatus(null);
          return;
        }
      } catch (e) {
        console.error("Ping failed:", e);
        setConnected(false);
        setStatus(null);
        return;
      }

      // Agent is alive, refresh data
      refreshStatus();
      refreshProcesses();
      refreshConnections();
      refreshFileOps();
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

  const refreshFileOps = async () => {
    try {
      const result = await invoke<FileOp[]>("get_file_ops", {});
      setFileOps(result);
    } catch (e) {
      console.error("Failed to get file ops:", e);
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

  // All processes (no sidebar filter needed anymore)
  const agentFilteredProcesses = processes;

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
        <ProcessTree
          processes={displayProcesses}
          selectedProcess={selectedProcess}
          onSelectProcess={setSelectedProcess}
        />
        <EventTable
          processes={processes}
          connections={connections}
          fileOps={fileOps}
        />
        {selectedProcess && (
          <DetailsPanel
            process={selectedProcess}
            connections={connections}
            fileOps={fileOps}
            onClose={() => setSelectedProcess(null)}
          />
        )}
      </div>
      <StatsBar
        processCount={processes.length}
        agentCount={agentProcesses.filter((a) => a.count > 0).length}
        eventsCollected={status?.eventsCollected ?? 0}
        fileOpsCount={fileOps.length}
        connectionsCount={connections.length}
      />
    </div>
  );
}

export default App;
