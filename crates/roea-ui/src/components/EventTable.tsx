import { useState, useEffect, useRef, useMemo } from "react";
import { FileText, Globe, Terminal, Pause, Play, ChevronDown, ChevronRight } from "lucide-react";
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
  FILE_WRITE: "var(--accent-primary)",
  FILE_DELETE: "var(--accent-danger)",
  NETWORK: "#a855f7",
  SPAWN: "var(--accent-success)",
  EXIT: "var(--text-muted)",
};

const severityColors: Record<string, string> = {
  info: "var(--text-secondary)",
  warning: "var(--accent-warning)",
  suspicious: "var(--accent-danger)",
};

function formatTime(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return date.toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
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
    if (fo.operation === "write" || fo.operation === "create") {
      eventType = "FILE_WRITE";
    } else if (fo.operation === "delete" || fo.operation === "unlink") {
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
    events.push({
      id: conn.id,
      timestamp: conn.timestamp,
      processId: conn.processId,
      pid: conn.pid,
      processName: process?.name || `PID ${conn.pid}`,
      agentType: process?.agentType || "unknown",
      eventType: "NETWORK",
      details: `${conn.remoteAddr}:${conn.remotePort} ${conn.state}`,
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
  const severityColor = severityColors[event.severity];

  return (
    <div className="event-row-container">
      <div
        className={`event-row ${isExpanded ? "expanded" : ""}`}
        onClick={onToggle}
        style={{ borderLeftColor: severityColor }}
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

        <span className="event-details">{event.details}</span>

        <span className="event-expand">
          {isExpanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
        </span>
      </div>

      {isExpanded && (
        <div className="event-expanded-details">
          <div className="event-detail-row">
            <span className="event-detail-label">Full Path:</span>
            <span className="event-detail-value monospace">{event.details}</span>
          </div>
          <div className="event-detail-row">
            <span className="event-detail-label">Process ID:</span>
            <span className="event-detail-value monospace">{event.processId}</span>
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

export function EventTable({
  processes,
  connections,
  fileOps,
}: EventTableProps) {
  const [isPaused, setIsPaused] = useState(false);
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set());
  const containerRef = useRef<HTMLDivElement>(null);

  const events = useMemo(
    () => generateEventsFromData(processes, connections, fileOps),
    [processes, connections, fileOps]
  );

  // Auto-scroll to top when new events arrive (unless paused)
  useEffect(() => {
    if (!isPaused && containerRef.current) {
      containerRef.current.scrollTop = 0;
    }
  }, [events, isPaused]);

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

  // Pause auto-scroll on hover
  const handleMouseEnter = () => setIsPaused(true);
  const handleMouseLeave = () => setIsPaused(false);

  return (
    <div className="event-table">
      <div className="event-header">
        <span className="event-title">Events</span>
        <span className="event-count">{events.length} events</span>
        <button
          className={`event-pause-btn ${isPaused ? "paused" : ""}`}
          onClick={() => setIsPaused(!isPaused)}
          title={isPaused ? "Resume auto-scroll" : "Pause auto-scroll"}
        >
          {isPaused ? <Play size={12} /> : <Pause size={12} />}
          {isPaused ? "Resume" : "Live"}
        </button>
      </div>

      <div className="event-table-header">
        <span className="event-col-time">Time</span>
        <span className="event-col-type">Type</span>
        <span className="event-col-process">Process</span>
        <span className="event-col-details">Details</span>
      </div>

      <div
        className="event-content"
        ref={containerRef}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
      >
        {events.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-icon">📋</div>
            <div className="empty-state-title">No events</div>
            <div className="empty-state-description">
              Events will appear here as processes perform actions
            </div>
          </div>
        ) : (
          events.map((event) => (
            <EventRow
              key={event.id}
              event={event}
              isExpanded={expandedIds.has(event.id)}
              onToggle={() => toggleExpanded(event.id)}
            />
          ))
        )}
      </div>
    </div>
  );
}
