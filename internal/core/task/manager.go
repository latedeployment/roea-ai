// Package task provides task management functionality.
package task

import (
	"fmt"
	"sync"
	"time"

	"github.com/roea-ai/roea/internal/crypto"
	"github.com/roea-ai/roea/internal/fossil"
	"github.com/roea-ai/roea/pkg/types"
)

// Manager handles task lifecycle operations.
type Manager struct {
	ticketStore    *fossil.TicketStore
	artifactStore  *fossil.ArtifactStore
	payloadService *crypto.PayloadService

	// Progress tracking
	progressMu      sync.RWMutex
	taskProgress    map[string]*types.TaskProgress
	progressWatchers map[string][]chan *types.TaskProgress
}

// NewManager creates a new task Manager.
func NewManager(
	ticketStore *fossil.TicketStore,
	artifactStore *fossil.ArtifactStore,
	payloadService *crypto.PayloadService,
) *Manager {
	return &Manager{
		ticketStore:      ticketStore,
		artifactStore:    artifactStore,
		payloadService:   payloadService,
		taskProgress:     make(map[string]*types.TaskProgress),
		progressWatchers: make(map[string][]chan *types.TaskProgress),
	}
}

// CreateTask creates a new task.
func (m *Manager) CreateTask(task *types.Task) error {
	// Set defaults
	if task.Status == "" {
		task.Status = types.TaskPending
	}
	if task.ExecutionMode == "" {
		task.ExecutionMode = types.ExecSingleShot
	}
	if task.Priority == 0 {
		task.Priority = 5
	}

	return m.ticketStore.CreateTask(task)
}

// GetTask retrieves a task by ID.
func (m *Manager) GetTask(id string) (*types.Task, error) {
	return m.ticketStore.GetTask(id)
}

// UpdateTask updates an existing task.
func (m *Manager) UpdateTask(id string, update *types.TaskUpdate) error {
	return m.ticketStore.UpdateTask(id, update)
}

// DeleteTask removes a task and its artifacts.
func (m *Manager) DeleteTask(id string) error {
	// Delete artifacts first
	if err := m.artifactStore.DeleteTaskArtifacts(id); err != nil {
		// Log but don't fail
	}

	return m.ticketStore.DeleteTask(id)
}

// ListTasks retrieves tasks matching the filter.
func (m *Manager) ListTasks(filter *types.TaskFilter) ([]*types.Task, error) {
	return m.ticketStore.ListTasks(filter)
}

// GetNextTask returns the next pending task for assignment.
func (m *Manager) GetNextTask(agentType string) (*types.Task, error) {
	return m.ticketStore.GetNextTask(agentType)
}

// AssignTask assigns a task to an agent and marks it as running.
func (m *Manager) AssignTask(taskID string, agentInstanceID string) error {
	now := time.Now()
	status := types.TaskRunning

	return m.ticketStore.UpdateTask(taskID, &types.TaskUpdate{
		Status:    &status,
		StartedAt: &now,
	})
}

// CompleteTask marks a task as completed.
func (m *Manager) CompleteTask(taskID string, result string, artifactPaths []string) error {
	now := time.Now()
	status := types.TaskCompleted

	// Store any artifacts
	// In a real implementation, we'd read each file and store it. For now, we
	// accept the list but do not persist it anywhere.
	_ = artifactPaths

	return m.ticketStore.UpdateTask(taskID, &types.TaskUpdate{
		Status:      &status,
		CompletedAt: &now,
		Result:      &result,
	})
}

// FailTask marks a task as failed.
func (m *Manager) FailTask(taskID string, errorMessage string) error {
	now := time.Now()
	status := types.TaskFailed

	return m.ticketStore.UpdateTask(taskID, &types.TaskUpdate{
		Status:       &status,
		CompletedAt:  &now,
		ErrorMessage: &errorMessage,
	})
}

// CancelTask marks a task as cancelled.
func (m *Manager) CancelTask(taskID string) error {
	now := time.Now()
	status := types.TaskCancelled

	return m.ticketStore.UpdateTask(taskID, &types.TaskUpdate{
		Status:      &status,
		CompletedAt: &now,
	})
}

// ReportProgress updates progress for a task.
func (m *Manager) ReportProgress(progress *types.TaskProgress) {
	m.progressMu.Lock()
	defer m.progressMu.Unlock()

	m.taskProgress[progress.TaskID] = progress

	// Notify watchers
	if watchers, ok := m.progressWatchers[progress.TaskID]; ok {
		for _, ch := range watchers {
			select {
			case ch <- progress:
			default:
				// Channel full, skip
			}
		}
	}
}

// GetProgress retrieves the current progress for a task.
func (m *Manager) GetProgress(taskID string) *types.TaskProgress {
	m.progressMu.RLock()
	defer m.progressMu.RUnlock()

	return m.taskProgress[taskID]
}

// WatchProgress returns a channel for progress updates.
func (m *Manager) WatchProgress(taskID string) (<-chan *types.TaskProgress, func()) {
	m.progressMu.Lock()
	defer m.progressMu.Unlock()

	ch := make(chan *types.TaskProgress, 10)
	m.progressWatchers[taskID] = append(m.progressWatchers[taskID], ch)

	// Return cleanup function
	cleanup := func() {
		m.progressMu.Lock()
		defer m.progressMu.Unlock()

		watchers := m.progressWatchers[taskID]
		for i, w := range watchers {
			if w == ch {
				m.progressWatchers[taskID] = append(watchers[:i], watchers[i+1:]...)
				break
			}
		}
		close(ch)
	}

	return ch, cleanup
}

// CreateSubtask creates a subtask linked to a parent.
func (m *Manager) CreateSubtask(parentID string, task *types.Task) error {
	task.ParentID = &parentID
	return m.CreateTask(task)
}

// GetSubtasks retrieves all subtasks of a parent task.
func (m *Manager) GetSubtasks(parentID string) ([]*types.Task, error) {
	return m.ticketStore.ListTasks(&types.TaskFilter{
		ParentID: &parentID,
	})
}

// GetTaskWithSecrets retrieves a task with decrypted secrets.
func (m *Manager) GetTaskWithSecrets(taskID string) (*types.Task, *types.TaskSecrets, error) {
	task, err := m.ticketStore.GetTask(taskID)
	if err != nil {
		return nil, nil, err
	}
	if task == nil {
		return nil, nil, fmt.Errorf("task not found: %s", taskID)
	}

	var secrets *types.TaskSecrets
	if task.Secrets != nil && m.payloadService != nil {
		secrets, err = m.payloadService.DecryptSecrets(task.Secrets)
		if err != nil {
			return task, nil, fmt.Errorf("failed to decrypt secrets: %w", err)
		}
	}

	return task, secrets, nil
}

// SetTaskSecrets encrypts and stores secrets for a task.
func (m *Manager) SetTaskSecrets(taskID string, secrets *types.TaskSecrets) error {
	if m.payloadService == nil {
		return fmt.Errorf("payload service not configured")
	}

	encrypted, err := m.payloadService.EncryptSecrets(secrets)
	if err != nil {
		return fmt.Errorf("failed to encrypt secrets: %w", err)
	}

	return m.ticketStore.UpdateTask(taskID, &types.TaskUpdate{
		Secrets: encrypted,
	})
}

// StoreArtifact stores an artifact for a task.
func (m *Manager) StoreArtifact(taskID string, name string, data []byte) error {
	return m.artifactStore.StoreArtifact(taskID, name, data)
}

// GetArtifact retrieves an artifact.
func (m *Manager) GetArtifact(taskID string, name string) ([]byte, string, error) {
	return m.artifactStore.GetArtifact(taskID, name)
}

// ListArtifacts lists all artifacts for a task.
func (m *Manager) ListArtifacts(taskID string) ([]*types.TaskArtifact, error) {
	return m.artifactStore.ListArtifacts(taskID)
}

// GetStats returns task statistics.
func (m *Manager) GetStats() (*TaskStats, error) {
	pending, err := m.ticketStore.CountTasks(&types.TaskFilter{
		Status: []types.TaskStatus{types.TaskPending},
	})
	if err != nil {
		return nil, err
	}

	running, err := m.ticketStore.CountTasks(&types.TaskFilter{
		Status: []types.TaskStatus{types.TaskRunning},
	})
	if err != nil {
		return nil, err
	}

	completed, err := m.ticketStore.CountTasks(&types.TaskFilter{
		Status: []types.TaskStatus{types.TaskCompleted},
	})
	if err != nil {
		return nil, err
	}

	failed, err := m.ticketStore.CountTasks(&types.TaskFilter{
		Status: []types.TaskStatus{types.TaskFailed},
	})
	if err != nil {
		return nil, err
	}

	return &TaskStats{
		Pending:   pending,
		Running:   running,
		Completed: completed,
		Failed:    failed,
		Total:     pending + running + completed + failed,
	}, nil
}

// TaskStats contains task count statistics.
type TaskStats struct {
	Pending   int `json:"pending"`
	Running   int `json:"running"`
	Completed int `json:"completed"`
	Failed    int `json:"failed"`
	Total     int `json:"total"`
}
