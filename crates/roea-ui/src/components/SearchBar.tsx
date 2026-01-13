import { useState, useCallback, useEffect } from "react";
import { Search, X, Download, Filter } from "lucide-react";
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
    filters.pidRange.max !== undefined;

  return (
    <div className="search-bar">
      <div className="search-input-container">
        <Search size={16} className="search-icon" />
        <input
          type="text"
          className="search-input"
          placeholder="Search processes, paths, PIDs..."
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
            <div className="filter-title">PID Range</div>
            <div className="filter-range">
              <input
                type="number"
                placeholder="Min"
                value={filters.pidRange.min ?? ""}
                onChange={(e) =>
                  setFilters((prev) => ({
                    ...prev,
                    pidRange: {
                      ...prev.pidRange,
                      min: e.target.value ? parseInt(e.target.value) : undefined,
                    },
                  }))
                }
              />
              <span>-</span>
              <input
                type="number"
                placeholder="Max"
                value={filters.pidRange.max ?? ""}
                onChange={(e) =>
                  setFilters((prev) => ({
                    ...prev,
                    pidRange: {
                      ...prev.pidRange,
                      max: e.target.value ? parseInt(e.target.value) : undefined,
                    },
                  }))
                }
              />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
