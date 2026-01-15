import { useState, useEffect, useRef, useMemo } from "react";
import {
  FileText,
  Globe,
  Terminal,
  Pause,
  Play,
  ChevronDown,
  ChevronRight,
  Filter,
} from "lucide-react";
import { Event, EventType, Process, Connection, FileOp } from "../lib/types";

interface EventTableProps {
  processes: Process[];
  connections: Connection[];
  fileOps: FileOp[];
}

const eventTypeIcons: Record<EventType, React.ReactNode> = {
  FILE_READ: <FileText size={12} />,
  FILE_WRITE: <FileText size={12} />,
  FILE_DELETE: <FileText size={12} />,
  NETWORK: <Globe size={12} />,
  SPAWN: <Terminal size={12} />,
  EXIT: <Terminal size={12} />,
};

const eventTypeColors: Record<EventType, string> = {
  FILE_READ: "var(--accent-primary)",
  FILE_WRITE: "#22c55e",
  FILE_DELETE: "var(--accent-danger)",
  NETWORK: "#a855f7",
  SPAWN: "#06b6d4",
  EXIT: "var(--text-muted)",
};

function formatTime(timestamp: number): string {
  if (!timestamp || timestamp <= 0) return "--:--:--";
  const date = new Date(timestamp * 1000);
  return date.toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

function formatNetworkDetails(conn: Connection): { display: string; isUnixSocket: boolean } {
  // Check if it's a unix socket (protocol contains "unix" or remote addr is a path)
  const isUnixSocket =
    conn.protocol?.toLowerCase().includes("unix") ||
    conn.remoteAddr?.startsWith("/") ||
    conn.localAddr?.startsWith("/") ||
    conn.remotePort === 0;

  if (isUnixSocket) {
    // For unix sockets, show the socket path
    const socketPath = conn.remoteAddr || conn.localAddr || "unix socket";
    return {
      display: `unix://${socketPath}`,
      isUnixSocket: true,
    };
  }

  // For TCP/UDP connections
  if (conn.remoteAddr && conn.remotePort) {
    return {
      display: `${conn.remoteAddr}:${conn.remotePort} [${conn.state || "UNKNOWN"}]`,
      isUnixSocket: false,
    };
  }

  // Listening socket
  if (conn.localAddr && conn.localPort) {
    return {
      display: `LISTEN ${conn.localAddr}:${conn.localPort}`,
      isUnixSocket: false,
    };
  }

  return {
    display: conn.state || "unknown",
    isUnixSocket: false,
  };
}

function generateEventsFromData(
  processes: Process[],
  connections: Connection[],
  fileOps: FileOp[]
): Event[] {
  const events: Event[] = [];
  const processMap = new Map<number, Process>();
  processes.forEach((p) => processMap.set(p.pid, p));

  // Convert file operations to events
  fileOps.forEach((fo) => {
    const process = processMap.get(fo.pid);
    let eventType: EventType = "FILE_READ";
    if (
      fo.operation === "write" ||
      fo.operation === "create" ||
      fo.operation === "WRITE" ||
      fo.operation === "CREATE"
    ) {
      eventType = "FILE_WRITE";
    } else if (
      fo.operation === "delete" ||
      fo.operation === "unlink" ||
      fo.operation === "DELETE" ||
      fo.operation === "UNLINK"
    ) {
      eventType = "FILE_DELETE";
    }
    events.push({
      id: fo.id,
      timestamp: fo.timestamp,
      processId: fo.processId,
      pid: fo.pid,
      processName: process?.name || `PID ${fo.pid}`,
      agentType: process?.agentType || "unknown",
      eventType,
      details: fo.path,
      severity: eventType === "FILE_DELETE" ? "warning" : "info",
    });
  });

  // Convert connections to events
  connections.forEach((conn) => {
    const process = processMap.get(conn.pid);
    const { display } = formatNetworkDetails(conn);
    events.push({
      id: conn.id,
      timestamp: conn.timestamp,
      processId: conn.processId,
      pid: conn.pid,
      processName: process?.name || `PID ${conn.pid}`,
      agentType: process?.agentType || "unknown",
      eventType: "NETWORK",
      details: display,
      severity: "info",
    });
  });

  // Convert process spawn/exit to events
  processes.forEach((p) => {
    if (p.startTime > 0) {
      events.push({
        id: `spawn-${p.id}`,
        timestamp: p.startTime,
        processId: p.id,
        pid: p.pid,
        processName: p.name,
        agentType: p.agentType,
        eventType: "SPAWN",
        details: p.cmdline.slice(0, 100) + (p.cmdline.length > 100 ? "..." : ""),
        severity: "info",
      });
    }
    if (p.endTime > 0) {
      events.push({
        id: `exit-${p.id}`,
        timestamp: p.endTime,
        processId: p.id,
        pid: p.pid,
        processName: p.name,
        agentType: p.agentType,
        eventType: "EXIT",
        details: "Process exited",
        severity: "info",
      });
    }
  });

  // Sort by timestamp descending (newest first)
  return events.sort((a, b) => b.timestamp - a.timestamp);
}

function EventRow({
  event,
  isExpanded,
  onToggle,
}: {
  event: Event;
  isExpanded: boolean;
  onToggle: () => void;
}) {
  const color = eventTypeColors[event.eventType];

  // Determine if this is a unix socket for special styling
  const isUnixSocket =
    event.eventType === "NETWORK" && event.details.startsWith("unix://");

  return (
    <div className="event-row-container">
      <div
        className={`event-row ${isExpanded ? "expanded" : ""}`}
        onClick={onToggle}
      >
        <span className="event-time">{formatTime(event.timestamp)}</span>

        <span className="event-type-badge" style={{ background: color }}>
          {eventTypeIcons[event.eventType]}
          <span>{event.eventType.replace("_", " ")}</span>
        </span>

        <span className="event-process">
          {event.processName}
          <span className="event-pid">({event.pid})</span>
        </span>

        <span className={`event-details ${isUnixSocket ? "unix-socket" : ""}`}>
          {event.details}
        </span>

        <span className="event-expand">
          {isExpanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
        </span>
      </div>

      {isExpanded && (
        <div className="event-expanded-details">
          <div className="event-detail-row">
            <span className="event-detail-label">Full Details:</span>
            <span className="event-detail-value monospace">{event.details}</span>
          </div>
          <div className="event-detail-row">
            <span className="event-detail-label">Process ID:</span>
            <span className="event-detail-value monospace">{event.processId}</span>
          </div>
          <div className="event-detail-row">
            <span className="event-detail-label">PID:</span>
            <span className="event-detail-value monospace">{event.pid}</span>
          </div>
          <div className="event-detail-row">
            <span className="event-detail-label">Agent Type:</span>
            <span className="event-detail-value">{event.agentType || "none"}</span>
          </div>
        </div>
      )}
    </div>
  );
}

interface EventFilters {
  showFiles: boolean;
  showNetwork: boolean;
  showProcesses: boolean;
  searchQuery: string;
}

export function EventTable({
  processes,
  connections,
  fileOps,
}: EventTableProps) {
  const [isLive, setIsLive] = useState(true);
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set());
  const [showFilters, setShowFilters] = useState(false);
  const [filters, setFilters] = useState<EventFilters>({
    showFiles: true,
    showNetwork: true,
    showProcesses: true,
    searchQuery: "",
  });
  const containerRef = useRef<HTMLDivElement>(null);

  // Generate all events
  const allEvents = useMemo(
    () => generateEventsFromData(processes, connections, fileOps),
    [processes, connections, fileOps]
  );

  // Filter events based on current filters
  const filteredEvents = useMemo(() => {
    return allEvents.filter((event) => {
      // Type filters
      if (
        !filters.showFiles &&
        (event.eventType === "FILE_READ" ||
          event.eventType === "FILE_WRITE" ||
          event.eventType === "FILE_DELETE")
      ) {
        return false;
      }
      if (!filters.showNetwork && event.eventType === "NETWORK") {
        return false;
      }
      if (
        !filters.showProcesses &&
        (event.eventType === "SPAWN" || event.eventType === "EXIT")
      ) {
        return false;
      }

      // Search query
      if (filters.searchQuery) {
        const query = filters.searchQuery.toLowerCase();
        const matchesProcess = event.processName.toLowerCase().includes(query);
        const matchesDetails = event.details.toLowerCase().includes(query);
        const matchesPid = event.pid.toString().includes(query);
        if (!matchesProcess && !matchesDetails && !matchesPid) {
          return false;
        }
      }

      return true;
    });
  }, [allEvents, filters]);

  // Auto-scroll to top when in live mode and new events arrive
  useEffect(() => {
    if (isLive && containerRef.current) {
      containerRef.current.scrollTop = 0;
    }
  }, [allEvents, isLive]);

  const toggleExpanded = (id: string) => {
    setExpandedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const hasActiveFilters =
    !filters.showFiles ||
    !filters.showNetwork ||
    !filters.showProcesses ||
    filters.searchQuery !== "";

  const displayEvents = filteredEvents;

  return (
    <div className="event-table">
      <div className="event-header">
        <span className="event-title">Live Events</span>
        <span className="event-count">
          {displayEvents.length}
          {hasActiveFilters && ` / ${allEvents.length}`}
        </span>

        <div className="event-controls">
          <button
            className={`event-filter-btn ${showFilters ? "active" : ""} ${hasActiveFilters ? "has-filters" : ""}`}
            onClick={() => setShowFilters(!showFilters)}
            title="Filter events"
          >
            <Filter size={12} />
            {hasActiveFilters && <span className="filter-indicator" />}
          </button>

          <button
            className={`event-live-btn ${isLive ? "live" : "paused"}`}
            onClick={() => setIsLive(!isLive)}
            title={isLive ? "Pause live updates" : "Resume live updates"}
          >
            {isLive ? <Pause size={12} /> : <Play size={12} />}
            {isLive ? "Live" : "Paused"}
          </button>
        </div>
      </div>

      {showFilters && (
        <div className="event-filters">
          <div className="event-filter-row">
            <label className="event-filter-checkbox">
              <input
                type="checkbox"
                checked={filters.showFiles}
                onChange={(e) =>
                  setFilters((prev) => ({ ...prev, showFiles: e.target.checked }))
                }
              />
              <FileText size={12} />
              <span>Files</span>
            </label>
            <label className="event-filter-checkbox">
              <input
                type="checkbox"
                checked={filters.showNetwork}
                onChange={(e) =>
                  setFilters((prev) => ({ ...prev, showNetwork: e.target.checked }))
                }
              />
              <Globe size={12} />
              <span>Network</span>
            </label>
            <label className="event-filter-checkbox">
              <input
                type="checkbox"
                checked={filters.showProcesses}
                onChange={(e) =>
                  setFilters((prev) => ({ ...prev, showProcesses: e.target.checked }))
                }
              />
              <Terminal size={12} />
              <span>Processes</span>
            </label>
          </div>
          <div className="event-filter-search">
            <input
              type="text"
              placeholder="Filter by process, path, IP..."
              value={filters.searchQuery}
              onChange={(e) =>
                setFilters((prev) => ({ ...prev, searchQuery: e.target.value }))
              }
            />
          </div>
        </div>
      )}

      <div className="event-table-header">
        <span className="event-col-time">Time</span>
        <span className="event-col-type">Type</span>
        <span className="event-col-process">Process</span>
        <span className="event-col-details">Details</span>
      </div>

      <div className="event-content" ref={containerRef}>
        {displayEvents.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-icon">📋</div>
            <div className="empty-state-title">
              {hasActiveFilters ? "No matching events" : "No events"}
            </div>
            <div className="empty-state-description">
              {hasActiveFilters
                ? "Try adjusting your filters"
                : "Events will appear here as processes perform actions"}
            </div>
          </div>
        ) : (
          displayEvents.slice(0, 500).map((event) => (
            <EventRow
              key={event.id}
              event={event}
              isExpanded={expandedIds.has(event.id)}
              onToggle={() => toggleExpanded(event.id)}
            />
          ))
        )}
        {displayEvents.length > 500 && (
          <div className="event-overflow-notice">
            Showing 500 of {displayEvents.length} events. Use filters to narrow
            down.
          </div>
        )}
      </div>
    </div>
  );
}
