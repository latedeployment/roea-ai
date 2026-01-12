package fossil

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"github.com/roea-ai/roea/pkg/types"
)

// ProcessEventStore handles process and event storage.
type ProcessEventStore struct {
	store *Store
}

// NewProcessEventStore creates a new ProcessEventStore.
func NewProcessEventStore(store *Store) *ProcessEventStore {
	return &ProcessEventStore{store: store}
}

// InitSchema creates the necessary tables for process tracking.
func (ps *ProcessEventStore) InitSchema() error {
	ps.store.mu.Lock()
	defer ps.store.mu.Unlock()

	// Create processes table
	_, err := ps.store.db.Exec(`
		CREATE TABLE IF NOT EXISTS processes (
			id TEXT PRIMARY KEY,
			pid INTEGER NOT NULL,
			parent_id TEXT,
			parent_pid INTEGER,
			task_id TEXT,
			instance_id TEXT,
			agent_type TEXT,
			command TEXT,
			args TEXT,
			status TEXT NOT NULL,
			exit_code INTEGER,
			started_at TEXT NOT NULL,
			ended_at TEXT,
			cpu_percent REAL DEFAULT 0,
			memory_bytes INTEGER DEFAULT 0,
			working_dir TEXT,
			is_agent_root INTEGER DEFAULT 0
		)
	`)
	if err != nil {
		return fmt.Errorf("failed to create processes table: %w", err)
	}

	// Create indexes for common queries
	indexes := []string{
		"CREATE INDEX IF NOT EXISTS idx_processes_task_id ON processes(task_id)",
		"CREATE INDEX IF NOT EXISTS idx_processes_instance_id ON processes(instance_id)",
		"CREATE INDEX IF NOT EXISTS idx_processes_parent_id ON processes(parent_id)",
		"CREATE INDEX IF NOT EXISTS idx_processes_status ON processes(status)",
		"CREATE INDEX IF NOT EXISTS idx_processes_pid ON processes(pid)",
	}

	for _, idx := range indexes {
		if _, err := ps.store.db.Exec(idx); err != nil {
			return fmt.Errorf("failed to create index: %w", err)
		}
	}

	// Create process_events table
	_, err = ps.store.db.Exec(`
		CREATE TABLE IF NOT EXISTS process_events (
			id TEXT PRIMARY KEY,
			process_id TEXT NOT NULL,
			pid INTEGER NOT NULL,
			task_id TEXT,
			instance_id TEXT,
			event_type TEXT NOT NULL,
			old_status TEXT,
			new_status TEXT,
			exit_code INTEGER,
			message TEXT,
			timestamp TEXT NOT NULL,
			FOREIGN KEY (process_id) REFERENCES processes(id)
		)
	`)
	if err != nil {
		return fmt.Errorf("failed to create process_events table: %w", err)
	}

	// Create indexes for events
	eventIndexes := []string{
		"CREATE INDEX IF NOT EXISTS idx_events_process_id ON process_events(process_id)",
		"CREATE INDEX IF NOT EXISTS idx_events_task_id ON process_events(task_id)",
		"CREATE INDEX IF NOT EXISTS idx_events_timestamp ON process_events(timestamp)",
	}

	for _, idx := range eventIndexes {
		if _, err := ps.store.db.Exec(idx); err != nil {
			return fmt.Errorf("failed to create event index: %w", err)
		}
	}

	return nil
}

// StoreProcess stores a new process record.
func (ps *ProcessEventStore) StoreProcess(process *types.ProcessNode) error {
	ps.store.mu.Lock()
	defer ps.store.mu.Unlock()

	argsJSON := ""
	if len(process.Args) > 0 {
		data, _ := json.Marshal(process.Args)
		argsJSON = string(data)
	}

	var endedAt sql.NullString
	if process.EndedAt != nil {
		endedAt = sql.NullString{String: process.EndedAt.Format(time.RFC3339), Valid: true}
	}

	var exitCode sql.NullInt64
	if process.ExitCode != nil {
		exitCode = sql.NullInt64{Int64: int64(*process.ExitCode), Valid: true}
	}

	isRoot := 0
	if process.IsAgentRoot {
		isRoot = 1
	}

	_, err := ps.store.db.Exec(`
		INSERT INTO processes (
			id, pid, parent_id, parent_pid, task_id, instance_id, agent_type,
			command, args, status, exit_code, started_at, ended_at,
			cpu_percent, memory_bytes, working_dir, is_agent_root
		) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
	`,
		process.ID,
		process.PID,
		process.ParentID,
		process.ParentPID,
		process.TaskID,
		process.InstanceID,
		process.AgentType,
		process.Command,
		argsJSON,
		string(process.Status),
		exitCode,
		process.StartedAt.Format(time.RFC3339),
		endedAt,
		process.CPUPercent,
		process.MemoryBytes,
		process.WorkingDir,
		isRoot,
	)

	if err != nil {
		return fmt.Errorf("failed to store process: %w", err)
	}

	return nil
}

// UpdateProcess updates an existing process record.
func (ps *ProcessEventStore) UpdateProcess(process *types.ProcessNode) error {
	ps.store.mu.Lock()
	defer ps.store.mu.Unlock()

	var endedAt sql.NullString
	if process.EndedAt != nil {
		endedAt = sql.NullString{String: process.EndedAt.Format(time.RFC3339), Valid: true}
	}

	var exitCode sql.NullInt64
	if process.ExitCode != nil {
		exitCode = sql.NullInt64{Int64: int64(*process.ExitCode), Valid: true}
	}

	result, err := ps.store.db.Exec(`
		UPDATE processes SET
			status = ?,
			exit_code = ?,
			ended_at = ?,
			cpu_percent = ?,
			memory_bytes = ?
		WHERE id = ?
	`,
		string(process.Status),
		exitCode,
		endedAt,
		process.CPUPercent,
		process.MemoryBytes,
		process.ID,
	)

	if err != nil {
		return fmt.Errorf("failed to update process: %w", err)
	}

	rows, _ := result.RowsAffected()
	if rows == 0 {
		return fmt.Errorf("process not found: %s", process.ID)
	}

	return nil
}

// GetProcess retrieves a process by ID.
func (ps *ProcessEventStore) GetProcess(id string) (*types.ProcessNode, error) {
	ps.store.mu.RLock()
	defer ps.store.mu.RUnlock()

	row := ps.store.db.QueryRow(`
		SELECT id, pid, parent_id, parent_pid, task_id, instance_id, agent_type,
			command, args, status, exit_code, started_at, ended_at,
			cpu_percent, memory_bytes, working_dir, is_agent_root
		FROM processes
		WHERE id = ?
	`, id)

	return ps.scanProcess(row)
}

// GetProcessByPID retrieves a process by OS PID.
func (ps *ProcessEventStore) GetProcessByPID(pid int) (*types.ProcessNode, error) {
	ps.store.mu.RLock()
	defer ps.store.mu.RUnlock()

	// Get the most recent process with this PID
	row := ps.store.db.QueryRow(`
		SELECT id, pid, parent_id, parent_pid, task_id, instance_id, agent_type,
			command, args, status, exit_code, started_at, ended_at,
			cpu_percent, memory_bytes, working_dir, is_agent_root
		FROM processes
		WHERE pid = ?
		ORDER BY started_at DESC
		LIMIT 1
	`, pid)

	return ps.scanProcess(row)
}

// ListProcesses retrieves processes matching the filter.
func (ps *ProcessEventStore) ListProcesses(filter *types.ProcessFilter) ([]*types.ProcessNode, error) {
	ps.store.mu.RLock()
	defer ps.store.mu.RUnlock()

	var whereClauses []string
	var args []interface{}

	if filter != nil {
		if filter.TaskID != "" {
			whereClauses = append(whereClauses, "task_id = ?")
			args = append(args, filter.TaskID)
		}

		if filter.InstanceID != "" {
			whereClauses = append(whereClauses, "instance_id = ?")
			args = append(args, filter.InstanceID)
		}

		if filter.ParentID != "" {
			whereClauses = append(whereClauses, "parent_id = ?")
			args = append(args, filter.ParentID)
		}

		if len(filter.Status) > 0 {
			placeholders := make([]string, len(filter.Status))
			for i, s := range filter.Status {
				placeholders[i] = "?"
				args = append(args, string(s))
			}
			whereClauses = append(whereClauses, fmt.Sprintf("status IN (%s)", strings.Join(placeholders, ",")))
		}

		if filter.IsRoot != nil {
			if *filter.IsRoot {
				whereClauses = append(whereClauses, "is_agent_root = 1")
			} else {
				whereClauses = append(whereClauses, "is_agent_root = 0")
			}
		}
	}

	query := `
		SELECT id, pid, parent_id, parent_pid, task_id, instance_id, agent_type,
			command, args, status, exit_code, started_at, ended_at,
			cpu_percent, memory_bytes, working_dir, is_agent_root
		FROM processes
	`

	if len(whereClauses) > 0 {
		query += " WHERE " + strings.Join(whereClauses, " AND ")
	}

	query += " ORDER BY started_at DESC"

	if filter != nil && filter.Limit > 0 {
		query += fmt.Sprintf(" LIMIT %d", filter.Limit)
		if filter.Offset > 0 {
			query += fmt.Sprintf(" OFFSET %d", filter.Offset)
		}
	}

	rows, err := ps.store.db.Query(query, args...)
	if err != nil {
		return nil, fmt.Errorf("failed to query processes: %w", err)
	}
	defer rows.Close()

	var processes []*types.ProcessNode
	for rows.Next() {
		process, err := ps.scanProcessRows(rows)
		if err != nil {
			return nil, err
		}
		processes = append(processes, process)
	}

	return processes, nil
}

// GetProcessTree retrieves a process tree starting from a root.
func (ps *ProcessEventStore) GetProcessTree(rootID string) (*types.ProcessTree, error) {
	root, err := ps.GetProcess(rootID)
	if err != nil {
		return nil, err
	}
	if root == nil {
		return nil, fmt.Errorf("process not found: %s", rootID)
	}

	tree := &types.ProcessTree{
		RootProcess: root,
	}

	// Get children recursively
	children, err := ps.ListProcesses(&types.ProcessFilter{ParentID: rootID})
	if err != nil {
		return tree, nil
	}

	for _, child := range children {
		childTree, err := ps.GetProcessTree(child.ID)
		if err != nil {
			continue
		}
		tree.Children = append(tree.Children, childTree)
	}

	return tree, nil
}

// GetProcessGraph returns data for graph visualization.
func (ps *ProcessEventStore) GetProcessGraph(filter *types.ProcessFilter) (*types.ProcessGraphData, error) {
	processes, err := ps.ListProcesses(filter)
	if err != nil {
		return nil, err
	}

	graph := &types.ProcessGraphData{
		Nodes: make([]types.ProcessGraphNode, 0, len(processes)),
		Edges: make([]types.ProcessGraphEdge, 0),
		Stats: types.ProcessStats{},
	}

	for _, proc := range processes {
		var memMB float64
		if proc.MemoryBytes > 0 {
			memMB = float64(proc.MemoryBytes) / (1024 * 1024)
		}

		var elapsedSecs int64
		if proc.EndedAt != nil {
			elapsedSecs = int64(proc.EndedAt.Sub(proc.StartedAt).Seconds())
		} else {
			elapsedSecs = int64(time.Since(proc.StartedAt).Seconds())
		}

		label := proc.AgentType
		if label == "" {
			label = proc.Command
		}
		if label == "" {
			label = fmt.Sprintf("PID %d", proc.PID)
		}

		graph.Nodes = append(graph.Nodes, types.ProcessGraphNode{
			ID:          proc.ID,
			Label:       label,
			PID:         proc.PID,
			AgentType:   proc.AgentType,
			TaskID:      proc.TaskID,
			InstanceID:  proc.InstanceID,
			Status:      proc.Status,
			IsRoot:      proc.IsAgentRoot,
			CPUPercent:  proc.CPUPercent,
			MemoryMB:    memMB,
			ElapsedSecs: elapsedSecs,
			StartedAt:   proc.StartedAt,
			EndedAt:     proc.EndedAt,
		})

		if proc.ParentID != "" {
			graph.Edges = append(graph.Edges, types.ProcessGraphEdge{
				ID:     fmt.Sprintf("%s->%s", proc.ParentID, proc.ID),
				Source: proc.ParentID,
				Target: proc.ID,
			})
		}

		// Update stats
		graph.Stats.TotalProcesses++
		if proc.Status == types.ProcessRunning || proc.Status == types.ProcessStarting {
			graph.Stats.RunningProcesses++
			graph.Stats.TotalMemoryBytes += proc.MemoryBytes
			graph.Stats.AvgCPUPercent += proc.CPUPercent
		}
		if proc.Status == types.ProcessCompleted {
			graph.Stats.CompletedCount++
		}
		if proc.Status == types.ProcessFailed {
			graph.Stats.FailedCount++
		}
	}

	if graph.Stats.RunningProcesses > 0 {
		graph.Stats.AvgCPUPercent /= float64(graph.Stats.RunningProcesses)
	}

	return graph, nil
}

// StoreEvent stores a process event.
func (ps *ProcessEventStore) StoreEvent(event *types.ProcessEvent) error {
	ps.store.mu.Lock()
	defer ps.store.mu.Unlock()

	var exitCode sql.NullInt64
	if event.ExitCode != nil {
		exitCode = sql.NullInt64{Int64: int64(*event.ExitCode), Valid: true}
	}

	_, err := ps.store.db.Exec(`
		INSERT INTO process_events (
			id, process_id, pid, task_id, instance_id, event_type,
			old_status, new_status, exit_code, message, timestamp
		) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
	`,
		event.ID,
		event.ProcessID,
		event.PID,
		event.TaskID,
		event.InstanceID,
		event.EventType,
		string(event.OldStatus),
		string(event.NewStatus),
		exitCode,
		event.Message,
		event.Timestamp.Format(time.RFC3339),
	)

	if err != nil {
		return fmt.Errorf("failed to store event: %w", err)
	}

	return nil
}

// GetEvents retrieves events for a process.
func (ps *ProcessEventStore) GetEvents(processID string, limit int) ([]*types.ProcessEvent, error) {
	ps.store.mu.RLock()
	defer ps.store.mu.RUnlock()

	query := `
		SELECT id, process_id, pid, task_id, instance_id, event_type,
			old_status, new_status, exit_code, message, timestamp
		FROM process_events
		WHERE process_id = ?
		ORDER BY timestamp DESC
	`

	if limit > 0 {
		query += fmt.Sprintf(" LIMIT %d", limit)
	}

	rows, err := ps.store.db.Query(query, processID)
	if err != nil {
		return nil, fmt.Errorf("failed to query events: %w", err)
	}
	defer rows.Close()

	var events []*types.ProcessEvent
	for rows.Next() {
		event, err := ps.scanEventRows(rows)
		if err != nil {
			return nil, err
		}
		events = append(events, event)
	}

	return events, nil
}

// DeleteOldProcesses removes processes older than maxAge.
func (ps *ProcessEventStore) DeleteOldProcesses(maxAge time.Duration) error {
	ps.store.mu.Lock()
	defer ps.store.mu.Unlock()

	cutoff := time.Now().Add(-maxAge).Format(time.RFC3339)

	// Delete events first (foreign key constraint)
	_, err := ps.store.db.Exec(`
		DELETE FROM process_events
		WHERE process_id IN (
			SELECT id FROM processes
			WHERE ended_at IS NOT NULL AND ended_at < ?
		)
	`, cutoff)
	if err != nil {
		return fmt.Errorf("failed to delete old events: %w", err)
	}

	// Delete processes
	_, err = ps.store.db.Exec(`
		DELETE FROM processes
		WHERE ended_at IS NOT NULL AND ended_at < ?
	`, cutoff)
	if err != nil {
		return fmt.Errorf("failed to delete old processes: %w", err)
	}

	return nil
}

// Scan helpers

func (ps *ProcessEventStore) scanProcess(row *sql.Row) (*types.ProcessNode, error) {
	var process types.ProcessNode
	var parentID, taskID, instanceID, agentType sql.NullString
	var command, argsJSON, status, workingDir sql.NullString
	var exitCode sql.NullInt64
	var startedAt, endedAt sql.NullString
	var cpuPercent sql.NullFloat64
	var memoryBytes sql.NullInt64
	var isRoot int

	err := row.Scan(
		&process.ID,
		&process.PID,
		&parentID,
		&process.ParentPID,
		&taskID,
		&instanceID,
		&agentType,
		&command,
		&argsJSON,
		&status,
		&exitCode,
		&startedAt,
		&endedAt,
		&cpuPercent,
		&memoryBytes,
		&workingDir,
		&isRoot,
	)

	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("failed to scan process: %w", err)
	}

	// Parse nullable fields
	if parentID.Valid {
		process.ParentID = parentID.String
	}
	if taskID.Valid {
		process.TaskID = taskID.String
	}
	if instanceID.Valid {
		process.InstanceID = instanceID.String
	}
	if agentType.Valid {
		process.AgentType = agentType.String
	}
	if command.Valid {
		process.Command = command.String
	}
	if argsJSON.Valid && argsJSON.String != "" {
		json.Unmarshal([]byte(argsJSON.String), &process.Args)
	}
	if status.Valid {
		process.Status = types.ProcessStatus(status.String)
	}
	if exitCode.Valid {
		code := int(exitCode.Int64)
		process.ExitCode = &code
	}
	if startedAt.Valid {
		t, _ := time.Parse(time.RFC3339, startedAt.String)
		process.StartedAt = t
	}
	if endedAt.Valid {
		t, _ := time.Parse(time.RFC3339, endedAt.String)
		process.EndedAt = &t
	}
	if cpuPercent.Valid {
		process.CPUPercent = cpuPercent.Float64
	}
	if memoryBytes.Valid {
		process.MemoryBytes = memoryBytes.Int64
	}
	if workingDir.Valid {
		process.WorkingDir = workingDir.String
	}
	process.IsAgentRoot = isRoot == 1

	return &process, nil
}

func (ps *ProcessEventStore) scanProcessRows(rows *sql.Rows) (*types.ProcessNode, error) {
	var process types.ProcessNode
	var parentID, taskID, instanceID, agentType sql.NullString
	var command, argsJSON, status, workingDir sql.NullString
	var exitCode sql.NullInt64
	var startedAt, endedAt sql.NullString
	var cpuPercent sql.NullFloat64
	var memoryBytes sql.NullInt64
	var isRoot int

	err := rows.Scan(
		&process.ID,
		&process.PID,
		&parentID,
		&process.ParentPID,
		&taskID,
		&instanceID,
		&agentType,
		&command,
		&argsJSON,
		&status,
		&exitCode,
		&startedAt,
		&endedAt,
		&cpuPercent,
		&memoryBytes,
		&workingDir,
		&isRoot,
	)

	if err != nil {
		return nil, fmt.Errorf("failed to scan process: %w", err)
	}

	// Parse nullable fields
	if parentID.Valid {
		process.ParentID = parentID.String
	}
	if taskID.Valid {
		process.TaskID = taskID.String
	}
	if instanceID.Valid {
		process.InstanceID = instanceID.String
	}
	if agentType.Valid {
		process.AgentType = agentType.String
	}
	if command.Valid {
		process.Command = command.String
	}
	if argsJSON.Valid && argsJSON.String != "" {
		json.Unmarshal([]byte(argsJSON.String), &process.Args)
	}
	if status.Valid {
		process.Status = types.ProcessStatus(status.String)
	}
	if exitCode.Valid {
		code := int(exitCode.Int64)
		process.ExitCode = &code
	}
	if startedAt.Valid {
		t, _ := time.Parse(time.RFC3339, startedAt.String)
		process.StartedAt = t
	}
	if endedAt.Valid {
		t, _ := time.Parse(time.RFC3339, endedAt.String)
		process.EndedAt = &t
	}
	if cpuPercent.Valid {
		process.CPUPercent = cpuPercent.Float64
	}
	if memoryBytes.Valid {
		process.MemoryBytes = memoryBytes.Int64
	}
	if workingDir.Valid {
		process.WorkingDir = workingDir.String
	}
	process.IsAgentRoot = isRoot == 1

	return &process, nil
}

func (ps *ProcessEventStore) scanEventRows(rows *sql.Rows) (*types.ProcessEvent, error) {
	var event types.ProcessEvent
	var taskID, instanceID sql.NullString
	var oldStatus, newStatus sql.NullString
	var exitCode sql.NullInt64
	var message sql.NullString
	var timestamp string

	err := rows.Scan(
		&event.ID,
		&event.ProcessID,
		&event.PID,
		&taskID,
		&instanceID,
		&event.EventType,
		&oldStatus,
		&newStatus,
		&exitCode,
		&message,
		&timestamp,
	)

	if err != nil {
		return nil, fmt.Errorf("failed to scan event: %w", err)
	}

	if taskID.Valid {
		event.TaskID = taskID.String
	}
	if instanceID.Valid {
		event.InstanceID = instanceID.String
	}
	if oldStatus.Valid {
		event.OldStatus = types.ProcessStatus(oldStatus.String)
	}
	if newStatus.Valid {
		event.NewStatus = types.ProcessStatus(newStatus.String)
	}
	if exitCode.Valid {
		code := int(exitCode.Int64)
		event.ExitCode = &code
	}
	if message.Valid {
		event.Message = message.String
	}
	event.Timestamp, _ = time.Parse(time.RFC3339, timestamp)

	return &event, nil
}
