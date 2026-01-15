import { useState, useCallback, useEffect } from "react";
import { Search, X, Download, Filter, FileText, Globe, Terminal } from "lucide-react";
import { Process } from "../lib/types";

interface SearchBarProps {
  processes: Process[];
  onFilteredProcesses: (filtered: Process[]) => void;
  onExport: (format: "json" | "csv") => void;
}

interface SearchFilters {
  query: string;
  agentTypes: string[];
  showActive: boolean;
  showExited: boolean;
  pidRange: { min?: number; max?: number };
  eventTypes: {
    files: boolean;
    network: boolean;
    processes: boolean;
  };
  pathFilter: string;
  ipFilter: string;
}

export function SearchBar({
  processes,
  onFilteredProcesses,
  onExport,
}: SearchBarProps) {
  const [filters, setFilters] = useState<SearchFilters>({
    query: "",
    agentTypes: [],
    showActive: true,
    showExited: true,
    pidRange: {},
    eventTypes: {
      files: true,
      network: true,
      processes: true,
    },
    pathFilter: "",
    ipFilter: "",
  });
  const [showAdvanced, setShowAdvanced] = useState(false);

  // Apply filters whenever they change
  useEffect(() => {
    const filtered = processes.filter((process) => {
      // Text search (name, cmdline, path)
      if (filters.query) {
        const query = filters.query.toLowerCase();
        const matchesName = process.name.toLowerCase().includes(query);
        const matchesCmdline = process.cmdline?.toLowerCase().includes(query);
        const matchesPath = process.exePath?.toLowerCase().includes(query);
        const matchesPid = process.pid.toString().includes(query);

        if (!matchesName && !matchesCmdline && !matchesPath && !matchesPid) {
          return false;
        }
      }

      // Agent type filter
      if (filters.agentTypes.length > 0) {
        if (!process.agentType || !filters.agentTypes.includes(process.agentType)) {
          return false;
        }
      }

      // Active/exited filter
      const isActive = process.endTime === 0;
      if (isActive && !filters.showActive) return false;
      if (!isActive && !filters.showExited) return false;

      // PID range
      if (filters.pidRange.min !== undefined && process.pid < filters.pidRange.min) {
        return false;
      }
      if (filters.pidRange.max !== undefined && process.pid > filters.pidRange.max) {
        return false;
      }

      return true;
    });

    onFilteredProcesses(filtered);
  }, [processes, filters, onFilteredProcesses]);

  const handleQueryChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    setFilters((prev) => ({ ...prev, query: e.target.value }));
  }, []);

  const clearSearch = useCallback(() => {
    setFilters({
      query: "",
      agentTypes: [],
      showActive: true,
      showExited: true,
      pidRange: {},
      eventTypes: {
        files: true,
        network: true,
        processes: true,
      },
      pathFilter: "",
      ipFilter: "",
    });
  }, []);

  const toggleAgentType = useCallback((agentType: string) => {
    setFilters((prev) => ({
      ...prev,
      agentTypes: prev.agentTypes.includes(agentType)
        ? prev.agentTypes.filter((t) => t !== agentType)
        : [...prev.agentTypes, agentType],
    }));
  }, []);

  // Get unique agent types from processes
  const agentTypes = [...new Set(processes.filter((p) => p.agentType).map((p) => p.agentType))];

  const hasActiveFilters =
    filters.query ||
    filters.agentTypes.length > 0 ||
    !filters.showActive ||
    !filters.showExited ||
    filters.pidRange.min !== undefined ||
    filters.pidRange.max !== undefined ||
    !filters.eventTypes.files ||
    !filters.eventTypes.network ||
    !filters.eventTypes.processes ||
    filters.pathFilter ||
    filters.ipFilter;

  return (
    <div className="search-bar">
      <div className="search-input-container">
        <Search size={16} className="search-icon" />
        <input
          type="text"
          className="search-input"
          placeholder="Search files, IPs, processes... (e.g., path:/home file:*.rs ip:142.250)"
          value={filters.query}
          onChange={handleQueryChange}
        />
        {hasActiveFilters && (
          <button className="search-clear" onClick={clearSearch} title="Clear filters">
            <X size={14} />
          </button>
        )}
      </div>

      <button
        className={`toolbar-button ${showAdvanced ? "active" : ""}`}
        onClick={() => setShowAdvanced(!showAdvanced)}
        title="Advanced filters"
      >
        <Filter size={14} />
        Filters
        {hasActiveFilters && <span className="filter-badge" />}
      </button>

      <div className="search-actions">
        <button
          className="toolbar-button"
          onClick={() => onExport("json")}
          title="Export as JSON"
        >
          <Download size={14} />
          JSON
        </button>
        <button
          className="toolbar-button"
          onClick={() => onExport("csv")}
          title="Export as CSV"
        >
          <Download size={14} />
          CSV
        </button>
      </div>

      {showAdvanced && (
        <div className="advanced-filters">
          <div className="filter-section">
            <div className="filter-title">Event Types</div>
            <div className="filter-options">
              <label className="filter-checkbox">
                <input
                  type="checkbox"
                  checked={filters.eventTypes.files}
                  onChange={(e) =>
                    setFilters((prev) => ({
                      ...prev,
                      eventTypes: { ...prev.eventTypes, files: e.target.checked },
                    }))
                  }
                />
                <FileText size={12} />
                <span>Files</span>
              </label>
              <label className="filter-checkbox">
                <input
                  type="checkbox"
                  checked={filters.eventTypes.network}
                  onChange={(e) =>
                    setFilters((prev) => ({
                      ...prev,
                      eventTypes: { ...prev.eventTypes, network: e.target.checked },
                    }))
                  }
                />
                <Globe size={12} />
                <span>Network</span>
              </label>
              <label className="filter-checkbox">
                <input
                  type="checkbox"
                  checked={filters.eventTypes.processes}
                  onChange={(e) =>
                    setFilters((prev) => ({
                      ...prev,
                      eventTypes: { ...prev.eventTypes, processes: e.target.checked },
                    }))
                  }
                />
                <Terminal size={12} />
                <span>Processes</span>
              </label>
            </div>
          </div>

          <div className="filter-section">
            <div className="filter-title">Agent Types</div>
            <div className="filter-options">
              {agentTypes.length > 0 ? (
                agentTypes.map((type) => (
                  <label key={type} className="filter-checkbox">
                    <input
                      type="checkbox"
                      checked={
                        filters.agentTypes.length === 0 ||
                        filters.agentTypes.includes(type!)
                      }
                      onChange={() => toggleAgentType(type!)}
                    />
                    <span>{type}</span>
                  </label>
                ))
              ) : (
                <span className="no-agents">No agents detected</span>
              )}
            </div>
          </div>

          <div className="filter-section">
            <div className="filter-title">Status</div>
            <div className="filter-options">
              <label className="filter-checkbox">
                <input
                  type="checkbox"
                  checked={filters.showActive}
                  onChange={(e) =>
                    setFilters((prev) => ({ ...prev, showActive: e.target.checked }))
                  }
                />
                <span>Active</span>
              </label>
              <label className="filter-checkbox">
                <input
                  type="checkbox"
                  checked={filters.showExited}
                  onChange={(e) =>
                    setFilters((prev) => ({ ...prev, showExited: e.target.checked }))
                  }
                />
                <span>Exited</span>
              </label>
            </div>
          </div>

          <div className="filter-section">
            <div className="filter-title">Path Filter</div>
            <div className="filter-input-group">
              <input
                type="text"
                className="filter-text-input"
                placeholder="/home/user/*.rs"
                value={filters.pathFilter}
                onChange={(e) =>
                  setFilters((prev) => ({ ...prev, pathFilter: e.target.value }))
                }
              />
            </div>
          </div>

          <div className="filter-section">
            <div className="filter-title">IP Filter</div>
            <div className="filter-input-group">
              <input
                type="text"
                className="filter-text-input"
                placeholder="142.250 or api.anthropic.com"
                value={filters.ipFilter}
                onChange={(e) =>
                  setFilters((prev) => ({ ...prev, ipFilter: e.target.value }))
                }
              />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
