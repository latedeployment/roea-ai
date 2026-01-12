// Package process provides process tracking and monitoring capabilities.
package process

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/roea-ai/roea/pkg/types"
)

// Tracker manages process tracking for all agent executions.
type Tracker struct {
	mu sync.RWMutex

	// Active processes indexed by process ID
	processes map[string]*types.ProcessNode

	// Process hierarchy: parent ID -> child IDs
	children map[string][]string

	// PID to process ID mapping
	pidToProcess map[int]string

	// Event subscribers
	subscribersMu sync.RWMutex
	subscribers   map[string]chan *types.ProcessEvent

	// Process event store (for persistence)
	eventStore ProcessEventStore

	// Background scanner
	scanInterval time.Duration
	stopCh       chan struct{}
}

// ProcessEventStore defines the interface for storing process events.
type ProcessEventStore interface {
	StoreEvent(event *types.ProcessEvent) error
	StoreProcess(process *types.ProcessNode) error
	UpdateProcess(process *types.ProcessNode) error
	GetProcess(id string) (*types.ProcessNode, error)
	GetProcessByPID(pid int) (*types.ProcessNode, error)
	ListProcesses(filter *types.ProcessFilter) ([]*types.ProcessNode, error)
	GetProcessTree(rootID string) (*types.ProcessTree, error)
	GetProcessGraph(filter *types.ProcessFilter) (*types.ProcessGraphData, error)
	GetEvents(processID string, limit int) ([]*types.ProcessEvent, error)
}

// NewTracker creates a new process tracker.
func NewTracker(eventStore ProcessEventStore) *Tracker {
	return &Tracker{
		processes:    make(map[string]*types.ProcessNode),
		children:     make(map[string][]string),
		pidToProcess: make(map[int]string),
		subscribers:  make(map[string]chan *types.ProcessEvent),
		eventStore:   eventStore,
		scanInterval: 1 * time.Second,
		stopCh:       make(chan struct{}),
	}
}

// Start begins background process monitoring.
func (t *Tracker) Start() {
	go t.scanLoop()
}

// Stop halts background monitoring.
func (t *Tracker) Stop() {
	close(t.stopCh)
}

// RegisterProcess registers a new process for tracking.
func (t *Tracker) RegisterProcess(node *types.ProcessNode) error {
	t.mu.Lock()
	defer t.mu.Unlock()

	// Generate ID if not provided
	if node.ID == "" {
		node.ID = generateProcessID()
	}

	// Set started time if not provided
	if node.StartedAt.IsZero() {
		node.StartedAt = time.Now()
	}

	// Set initial status
	if node.Status == "" {
		node.Status = types.ProcessStarting
	}

	// Store in memory
	t.processes[node.ID] = node
	t.pidToProcess[node.PID] = node.ID

	// Track parent-child relationship
	if node.ParentID != "" {
		t.children[node.ParentID] = append(t.children[node.ParentID], node.ID)
	}

	// Persist
	if t.eventStore != nil {
		if err := t.eventStore.StoreProcess(node); err != nil {
			return fmt.Errorf("failed to store process: %w", err)
		}
	}

	// Emit event
	t.emitEvent(&types.ProcessEvent{
		ID:         generateEventID(),
		ProcessID:  node.ID,
		PID:        node.PID,
		TaskID:     node.TaskID,
		InstanceID: node.InstanceID,
		EventType:  "started",
		NewStatus:  node.Status,
		Timestamp:  time.Now(),
	})

	return nil
}

// UpdateProcessStatus updates the status of a tracked process.
func (t *Tracker) UpdateProcessStatus(processID string, status types.ProcessStatus, exitCode *int) error {
	t.mu.Lock()
	defer t.mu.Unlock()

	node, ok := t.processes[processID]
	if !ok {
		return fmt.Errorf("process not found: %s", processID)
	}

	oldStatus := node.Status
	node.Status = status

	if exitCode != nil {
		node.ExitCode = exitCode
	}

	if status == types.ProcessCompleted || status == types.ProcessFailed || status == types.ProcessTerminated {
		now := time.Now()
		node.EndedAt = &now
	}

	// Update in store
	if t.eventStore != nil {
		if err := t.eventStore.UpdateProcess(node); err != nil {
			return fmt.Errorf("failed to update process: %w", err)
		}
	}

	// Emit event
	t.emitEvent(&types.ProcessEvent{
		ID:         generateEventID(),
		ProcessID:  node.ID,
		PID:        node.PID,
		TaskID:     node.TaskID,
		InstanceID: node.InstanceID,
		EventType:  "status_change",
		OldStatus:  oldStatus,
		NewStatus:  status,
		ExitCode:   exitCode,
		Timestamp:  time.Now(),
	})

	return nil
}

// UpdateProcessByPID updates process status by OS PID.
func (t *Tracker) UpdateProcessByPID(pid int, status types.ProcessStatus, exitCode *int) error {
	t.mu.RLock()
	processID, ok := t.pidToProcess[pid]
	t.mu.RUnlock()

	if !ok {
		return fmt.Errorf("process with PID %d not found", pid)
	}

	return t.UpdateProcessStatus(processID, status, exitCode)
}

// GetProcess returns a process by ID.
func (t *Tracker) GetProcess(id string) (*types.ProcessNode, error) {
	t.mu.RLock()
	node, ok := t.processes[id]
	t.mu.RUnlock()

	if ok {
		return node, nil
	}

	// Try from store
	if t.eventStore != nil {
		return t.eventStore.GetProcess(id)
	}

	return nil, fmt.Errorf("process not found: %s", id)
}

// GetProcessByPID returns a process by OS PID.
func (t *Tracker) GetProcessByPID(pid int) (*types.ProcessNode, error) {
	t.mu.RLock()
	processID, ok := t.pidToProcess[pid]
	t.mu.RUnlock()

	if ok {
		return t.GetProcess(processID)
	}

	// Try from store
	if t.eventStore != nil {
		return t.eventStore.GetProcessByPID(pid)
	}

	return nil, fmt.Errorf("process with PID %d not found", pid)
}

// ListProcesses returns processes matching the filter.
func (t *Tracker) ListProcesses(filter *types.ProcessFilter) ([]*types.ProcessNode, error) {
	t.mu.RLock()
	defer t.mu.RUnlock()

	var result []*types.ProcessNode

	for _, node := range t.processes {
		if matchesFilter(node, filter) {
			result = append(result, node)
		}
	}

	// Also get from store for historical data
	if t.eventStore != nil {
		stored, err := t.eventStore.ListProcesses(filter)
		if err == nil {
			// Merge, preferring in-memory data
			seen := make(map[string]bool)
			for _, n := range result {
				seen[n.ID] = true
			}
			for _, n := range stored {
				if !seen[n.ID] {
					result = append(result, n)
				}
			}
		}
	}

	return result, nil
}

// GetProcessTree returns the process tree for a root process.
func (t *Tracker) GetProcessTree(rootID string) (*types.ProcessTree, error) {
	t.mu.RLock()
	defer t.mu.RUnlock()

	return t.buildTree(rootID)
}

// GetProcessGraph returns data for graph visualization.
func (t *Tracker) GetProcessGraph(filter *types.ProcessFilter) (*types.ProcessGraphData, error) {
	processes, err := t.ListProcesses(filter)
	if err != nil {
		return nil, err
	}

	graph := &types.ProcessGraphData{
		Nodes: make([]types.ProcessGraphNode, 0, len(processes)),
		Edges: make([]types.ProcessGraphEdge, 0),
		Stats: types.ProcessStats{},
	}

	for _, proc := range processes {
		// Build node
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

		// Build edge if has parent
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

	// Calculate average CPU
	if graph.Stats.RunningProcesses > 0 {
		graph.Stats.AvgCPUPercent /= float64(graph.Stats.RunningProcesses)
	}

	return graph, nil
}

// GetActiveProcesses returns all currently running processes.
func (t *Tracker) GetActiveProcesses() []*types.ProcessNode {
	t.mu.RLock()
	defer t.mu.RUnlock()

	var active []*types.ProcessNode
	for _, node := range t.processes {
		if node.Status == types.ProcessRunning || node.Status == types.ProcessStarting {
			active = append(active, node)
		}
	}
	return active
}

// Subscribe creates a new event subscription.
func (t *Tracker) Subscribe(id string) <-chan *types.ProcessEvent {
	t.subscribersMu.Lock()
	defer t.subscribersMu.Unlock()

	ch := make(chan *types.ProcessEvent, 100)
	t.subscribers[id] = ch
	return ch
}

// Unsubscribe removes an event subscription.
func (t *Tracker) Unsubscribe(id string) {
	t.subscribersMu.Lock()
	defer t.subscribersMu.Unlock()

	if ch, ok := t.subscribers[id]; ok {
		close(ch)
		delete(t.subscribers, id)
	}
}

// DiscoverChildProcesses finds child processes of a given PID.
func (t *Tracker) DiscoverChildProcesses(parentPID int, taskID, instanceID, agentType string) error {
	children, err := getChildPIDs(parentPID)
	if err != nil {
		return err
	}

	t.mu.RLock()
	parentProcessID := t.pidToProcess[parentPID]
	t.mu.RUnlock()

	for _, childPID := range children {
		// Check if already tracked
		t.mu.RLock()
		_, exists := t.pidToProcess[childPID]
		t.mu.RUnlock()

		if exists {
			continue
		}

		// Get process info
		info, err := getProcessInfo(childPID)
		if err != nil {
			continue
		}

		node := &types.ProcessNode{
			ID:          generateProcessID(),
			PID:         childPID,
			ParentID:    parentProcessID,
			ParentPID:   parentPID,
			TaskID:      taskID,
			InstanceID:  instanceID,
			AgentType:   agentType,
			Command:     info.Command,
			Args:        info.Args,
			Status:      types.ProcessRunning,
			StartedAt:   time.Now(),
			WorkingDir:  info.WorkingDir,
			IsAgentRoot: false,
		}

		t.RegisterProcess(node)

		// Recursively discover children
		t.DiscoverChildProcesses(childPID, taskID, instanceID, agentType)
	}

	return nil
}

// CleanupEndedProcesses removes ended processes from memory (keeps in store).
func (t *Tracker) CleanupEndedProcesses(maxAge time.Duration) {
	t.mu.Lock()
	defer t.mu.Unlock()

	cutoff := time.Now().Add(-maxAge)

	for id, node := range t.processes {
		if node.EndedAt != nil && node.EndedAt.Before(cutoff) {
			delete(t.processes, id)
			delete(t.pidToProcess, node.PID)

			// Remove from children map
			if node.ParentID != "" {
				children := t.children[node.ParentID]
				for i, childID := range children {
					if childID == id {
						t.children[node.ParentID] = append(children[:i], children[i+1:]...)
						break
					}
				}
			}
		}
	}
}

// Internal methods

func (t *Tracker) buildTree(rootID string) (*types.ProcessTree, error) {
	node, ok := t.processes[rootID]
	if !ok {
		return nil, fmt.Errorf("process not found: %s", rootID)
	}

	tree := &types.ProcessTree{
		RootProcess: node,
	}

	childIDs := t.children[rootID]
	for _, childID := range childIDs {
		childTree, err := t.buildTree(childID)
		if err != nil {
			continue
		}
		tree.Children = append(tree.Children, childTree)
	}

	return tree, nil
}

func (t *Tracker) emitEvent(event *types.ProcessEvent) {
	// Store event
	if t.eventStore != nil {
		t.eventStore.StoreEvent(event)
	}

	// Notify subscribers
	t.subscribersMu.RLock()
	defer t.subscribersMu.RUnlock()

	for _, ch := range t.subscribers {
		select {
		case ch <- event:
		default:
			// Channel full, skip
		}
	}
}

func (t *Tracker) scanLoop() {
	ticker := time.NewTicker(t.scanInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			t.scanProcesses()
		case <-t.stopCh:
			return
		}
	}
}

func (t *Tracker) scanProcesses() {
	t.mu.Lock()
	defer t.mu.Unlock()

	for id, node := range t.processes {
		if node.Status != types.ProcessRunning && node.Status != types.ProcessStarting {
			continue
		}

		// Check if process is still alive
		alive := isProcessAlive(node.PID)

		if !alive {
			// Process has ended
			exitCode := getExitCode(node.PID)
			oldStatus := node.Status

			if exitCode != nil && *exitCode == 0 {
				node.Status = types.ProcessCompleted
			} else {
				node.Status = types.ProcessFailed
			}
			node.ExitCode = exitCode
			now := time.Now()
			node.EndedAt = &now

			// Update store
			if t.eventStore != nil {
				t.eventStore.UpdateProcess(node)
			}

			// Emit event
			t.emitEvent(&types.ProcessEvent{
				ID:         generateEventID(),
				ProcessID:  id,
				PID:        node.PID,
				TaskID:     node.TaskID,
				InstanceID: node.InstanceID,
				EventType:  "ended",
				OldStatus:  oldStatus,
				NewStatus:  node.Status,
				ExitCode:   exitCode,
				Timestamp:  time.Now(),
			})
		} else {
			// Update status to running if still starting
			if node.Status == types.ProcessStarting {
				node.Status = types.ProcessRunning
				if t.eventStore != nil {
					t.eventStore.UpdateProcess(node)
				}
			}

			// Update resource usage
			stats, err := getProcessStats(node.PID)
			if err == nil {
				node.CPUPercent = stats.CPUPercent
				node.MemoryBytes = stats.MemoryBytes
			}

			// Discover new children
			go func(pid int, taskID, instanceID, agentType string) {
				t.DiscoverChildProcesses(pid, taskID, instanceID, agentType)
			}(node.PID, node.TaskID, node.InstanceID, node.AgentType)
		}
	}
}

func matchesFilter(node *types.ProcessNode, filter *types.ProcessFilter) bool {
	if filter == nil {
		return true
	}

	if filter.TaskID != "" && node.TaskID != filter.TaskID {
		return false
	}

	if filter.InstanceID != "" && node.InstanceID != filter.InstanceID {
		return false
	}

	if filter.ParentID != "" && node.ParentID != filter.ParentID {
		return false
	}

	if len(filter.Status) > 0 {
		found := false
		for _, s := range filter.Status {
			if node.Status == s {
				found = true
				break
			}
		}
		if !found {
			return false
		}
	}

	if filter.IsRoot != nil && node.IsAgentRoot != *filter.IsRoot {
		return false
	}

	return true
}

// Helper types and functions

type processInfo struct {
	Command    string
	Args       []string
	WorkingDir string
}

type processStats struct {
	CPUPercent  float64
	MemoryBytes int64
}

func generateProcessID() string {
	return fmt.Sprintf("proc_%x", time.Now().UnixNano())
}

func generateEventID() string {
	return fmt.Sprintf("evt_%x", time.Now().UnixNano())
}

// Linux-specific process utilities

func isProcessAlive(pid int) bool {
	proc, err := os.FindProcess(pid)
	if err != nil {
		return false
	}
	// On Unix, FindProcess always succeeds, so we need to send signal 0
	err = proc.Signal(os.Signal(nil))
	return err == nil
}

func getExitCode(pid int) *int {
	// On Linux, we can't easily get exit code after process exits
	// unless we're the parent. Return nil for unknown.
	return nil
}

func getChildPIDs(parentPID int) ([]int, error) {
	// Read /proc/<pid>/task/<tid>/children for each thread
	// Or parse /proc/<pid>/status for child threads
	// Simpler approach: scan /proc for processes with matching PPID

	entries, err := os.ReadDir("/proc")
	if err != nil {
		return nil, err
	}

	var children []int

	for _, entry := range entries {
		if !entry.IsDir() {
			continue
		}

		pid, err := strconv.Atoi(entry.Name())
		if err != nil {
			continue
		}

		statusPath := filepath.Join("/proc", entry.Name(), "status")
		data, err := os.ReadFile(statusPath)
		if err != nil {
			continue
		}

		// Parse PPid line
		for _, line := range strings.Split(string(data), "\n") {
			if strings.HasPrefix(line, "PPid:") {
				parts := strings.Fields(line)
				if len(parts) >= 2 {
					ppid, err := strconv.Atoi(parts[1])
					if err == nil && ppid == parentPID {
						children = append(children, pid)
					}
				}
				break
			}
		}
	}

	return children, nil
}

func getProcessInfo(pid int) (*processInfo, error) {
	cmdlinePath := filepath.Join("/proc", strconv.Itoa(pid), "cmdline")
	data, err := os.ReadFile(cmdlinePath)
	if err != nil {
		return nil, err
	}

	// cmdline is null-separated
	parts := strings.Split(strings.TrimRight(string(data), "\x00"), "\x00")
	if len(parts) == 0 {
		return nil, fmt.Errorf("empty cmdline for pid %d", pid)
	}

	cwdPath := filepath.Join("/proc", strconv.Itoa(pid), "cwd")
	cwd, _ := os.Readlink(cwdPath)

	info := &processInfo{
		Command:    parts[0],
		WorkingDir: cwd,
	}

	if len(parts) > 1 {
		info.Args = parts[1:]
	}

	return info, nil
}

func getProcessStats(pid int) (*processStats, error) {
	statPath := filepath.Join("/proc", strconv.Itoa(pid), "stat")
	data, err := os.ReadFile(statPath)
	if err != nil {
		return nil, err
	}

	// Parse /proc/<pid>/stat - fields are space-separated
	// Field 14: utime, Field 15: stime, Field 23: vsize, Field 24: rss
	fields := strings.Fields(string(data))
	if len(fields) < 24 {
		return nil, fmt.Errorf("invalid stat format")
	}

	// Get memory from statm for more accuracy
	statmPath := filepath.Join("/proc", strconv.Itoa(pid), "statm")
	statmData, err := os.ReadFile(statmPath)
	if err != nil {
		return nil, err
	}

	statmFields := strings.Fields(string(statmData))
	if len(statmFields) < 2 {
		return nil, fmt.Errorf("invalid statm format")
	}

	// RSS is in pages (usually 4KB)
	rssPages, _ := strconv.ParseInt(statmFields[1], 10, 64)
	pageSize := int64(os.Getpagesize())

	stats := &processStats{
		MemoryBytes: rssPages * pageSize,
	}

	// CPU calculation would require sampling over time
	// For now, just return memory stats
	// TODO: Implement proper CPU tracking with time-based sampling

	return stats, nil
}

// GetProcessesByTask returns all processes for a specific task.
func (t *Tracker) GetProcessesByTask(taskID string) []*types.ProcessNode {
	filter := &types.ProcessFilter{TaskID: taskID}
	processes, _ := t.ListProcesses(filter)
	return processes
}

// GetProcessesByInstance returns all processes for a specific agent instance.
func (t *Tracker) GetProcessesByInstance(instanceID string) []*types.ProcessNode {
	filter := &types.ProcessFilter{InstanceID: instanceID}
	processes, _ := t.ListProcesses(filter)
	return processes
}

// TerminateProcess terminates a process and all its children.
func (t *Tracker) TerminateProcess(processID string) error {
	node, err := t.GetProcess(processID)
	if err != nil {
		return err
	}

	// Terminate children first
	t.mu.RLock()
	childIDs := t.children[processID]
	t.mu.RUnlock()

	for _, childID := range childIDs {
		t.TerminateProcess(childID)
	}

	// Terminate this process
	proc, err := os.FindProcess(node.PID)
	if err != nil {
		return err
	}

	if err := proc.Kill(); err != nil {
		return err
	}

	// Update status
	t.UpdateProcessStatus(processID, types.ProcessTerminated, nil)

	return nil
}

// ReadProcessOutput reads the output of a process if available.
func ReadProcessOutput(pid int) (string, error) {
	// Try to read from /proc/<pid>/fd/1 (stdout) - usually requires same user/root
	fdPath := filepath.Join("/proc", strconv.Itoa(pid), "fd", "1")
	target, err := os.Readlink(fdPath)
	if err != nil {
		return "", err
	}

	// If stdout is a file, we can read it
	if strings.HasPrefix(target, "/") && !strings.HasPrefix(target, "pipe:") {
		data, err := os.ReadFile(target)
		if err != nil {
			return "", err
		}
		return string(data), nil
	}

	return "", fmt.Errorf("stdout is not a file: %s", target)
}

// WatchProcessOutput streams output from a process.
func WatchProcessOutput(pid int, callback func(line string)) error {
	fdPath := filepath.Join("/proc", strconv.Itoa(pid), "fd", "1")
	target, err := os.Readlink(fdPath)
	if err != nil {
		return err
	}

	// Only works if stdout is a file
	if !strings.HasPrefix(target, "/") || strings.HasPrefix(target, "pipe:") {
		return fmt.Errorf("stdout is not a regular file")
	}

	file, err := os.Open(target)
	if err != nil {
		return err
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		callback(scanner.Text())
	}

	return scanner.Err()
}
