package types

import "time"

// ProcessStatus represents the state of a tracked process.
type ProcessStatus string

const (
	ProcessStarting   ProcessStatus = "starting"
	ProcessRunning    ProcessStatus = "running"
	ProcessCompleted  ProcessStatus = "completed"
	ProcessFailed     ProcessStatus = "failed"
	ProcessTerminated ProcessStatus = "terminated"
)

// ProcessNode represents a single process in the process tree.
type ProcessNode struct {
	ID           string        `json:"id"`            // Unique identifier (UUID)
	PID          int           `json:"pid"`           // OS process ID
	ParentID     string        `json:"parent_id"`     // Parent ProcessNode ID (empty if root)
	ParentPID    int           `json:"parent_pid"`    // Parent OS PID
	TaskID       string        `json:"task_id"`       // Associated task ID
	InstanceID   string        `json:"instance_id"`   // Agent instance ID
	AgentType    string        `json:"agent_type"`    // Type of agent (e.g., "general-coder")
	Command      string        `json:"command"`       // Command that started the process
	Args         []string      `json:"args"`          // Command arguments
	Status       ProcessStatus `json:"status"`        // Current status
	ExitCode     *int          `json:"exit_code"`     // Exit code (nil if still running)
	StartedAt    time.Time     `json:"started_at"`    // When process started
	EndedAt      *time.Time    `json:"ended_at"`      // When process ended (nil if running)
	CPUPercent   float64       `json:"cpu_percent"`   // Current CPU usage
	MemoryBytes  int64         `json:"memory_bytes"`  // Current memory usage
	WorkingDir   string        `json:"working_dir"`   // Working directory
	IsAgentRoot  bool          `json:"is_agent_root"` // True if this is the root agent process
}

// ProcessTree represents the full process hierarchy for a task or instance.
type ProcessTree struct {
	RootProcess *ProcessNode   `json:"root_process"`
	Children    []*ProcessTree `json:"children,omitempty"`
}

// ProcessEvent represents a state change event for a process.
type ProcessEvent struct {
	ID         string        `json:"id"`          // Event ID
	ProcessID  string        `json:"process_id"`  // Process node ID
	PID        int           `json:"pid"`         // OS process ID
	TaskID     string        `json:"task_id"`     // Associated task
	InstanceID string        `json:"instance_id"` // Agent instance
	EventType  string        `json:"event_type"`  // "started", "status_change", "output", "ended"
	OldStatus  ProcessStatus `json:"old_status"`  // Previous status
	NewStatus  ProcessStatus `json:"new_status"`  // New status
	ExitCode   *int          `json:"exit_code"`   // Exit code if ended
	Message    string        `json:"message"`     // Event message/output
	Timestamp  time.Time     `json:"timestamp"`   // When event occurred
}

// ProcessStats contains aggregated statistics for process tracking.
type ProcessStats struct {
	TotalProcesses   int     `json:"total_processes"`
	RunningProcesses int     `json:"running_processes"`
	CompletedCount   int     `json:"completed_count"`
	FailedCount      int     `json:"failed_count"`
	AvgCPUPercent    float64 `json:"avg_cpu_percent"`
	TotalMemoryBytes int64   `json:"total_memory_bytes"`
}

// ProcessFilter defines criteria for filtering processes.
type ProcessFilter struct {
	TaskID     string          `json:"task_id,omitempty"`
	InstanceID string          `json:"instance_id,omitempty"`
	ParentID   string          `json:"parent_id,omitempty"`
	Status     []ProcessStatus `json:"status,omitempty"`
	IsRoot     *bool           `json:"is_root,omitempty"`
	Limit      int             `json:"limit,omitempty"`
	Offset     int             `json:"offset,omitempty"`
}

// ProcessGraphData represents the data needed to render a process graph in the UI.
type ProcessGraphData struct {
	Nodes []ProcessGraphNode `json:"nodes"`
	Edges []ProcessGraphEdge `json:"edges"`
	Stats ProcessStats       `json:"stats"`
}

// ProcessGraphNode represents a node in the process graph visualization.
type ProcessGraphNode struct {
	ID          string        `json:"id"`
	Label       string        `json:"label"`
	PID         int           `json:"pid"`
	AgentType   string        `json:"agent_type"`
	TaskID      string        `json:"task_id"`
	InstanceID  string        `json:"instance_id"`
	Status      ProcessStatus `json:"status"`
	IsRoot      bool          `json:"is_root"`
	CPUPercent  float64       `json:"cpu_percent"`
	MemoryMB    float64       `json:"memory_mb"`
	ElapsedSecs int64         `json:"elapsed_secs"`
	StartedAt   time.Time     `json:"started_at"`
	EndedAt     *time.Time    `json:"ended_at"`
}

// ProcessGraphEdge represents a parent-child relationship in the graph.
type ProcessGraphEdge struct {
	ID     string `json:"id"`
	Source string `json:"source"` // Parent process ID
	Target string `json:"target"` // Child process ID
}

// WebSocketMessage represents a message sent over WebSocket for real-time updates.
type WebSocketMessage struct {
	Type    string      `json:"type"`    // "process_update", "process_event", "stats_update"
	Payload interface{} `json:"payload"` // The actual data
}

// ProcessUpdatePayload is sent when a process state changes.
type ProcessUpdatePayload struct {
	Process *ProcessNode `json:"process"`
	Event   string       `json:"event"` // "created", "updated", "deleted"
}
