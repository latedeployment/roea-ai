// Package types provides shared type definitions for the Roea AI system.
package types

import (
	"time"
)

// TaskStatus represents the current state of a task.
type TaskStatus string

const (
	TaskPending   TaskStatus = "pending"   // Open
	TaskAssigned  TaskStatus = "assigned"  // Assigned to an agent
	TaskRunning   TaskStatus = "running"   // In Progress
	TaskCompleted TaskStatus = "completed" // Closed successfully
	TaskFailed    TaskStatus = "failed"    // Closed (failed)
	TaskCancelled TaskStatus = "cancelled" // Closed (cancelled)
)

// ExecutionMode defines how an agent executes tasks.
type ExecutionMode string

const (
	// ExecRalphWiggum is a bash loop where agent picks tasks and outputs END to terminate.
	ExecRalphWiggum ExecutionMode = "ralph_wiggum"
	// ExecSpecFlow is a structured workflow with defined stages.
	ExecSpecFlow ExecutionMode = "spec_flow"
	// ExecSingleShot is one task, one execution.
	ExecSingleShot ExecutionMode = "single_shot"
	// ExecContinuous is a long-running agent monitoring queue.
	ExecContinuous ExecutionMode = "continuous"
)

// Task represents a unit of work to be executed by an agent.
// Maps to a Fossil ticket.
type Task struct {
	ID            string            `json:"id"`                       // Fossil ticket UUID
	Title         string            `json:"title"`                    // ticket.title
	Description   string            `json:"description"`              // ticket.comment
	Status        TaskStatus        `json:"status"`                   // ticket.status
	AgentType     string            `json:"agent_type"`               // ticket.type (repurposed)
	ExecutionMode ExecutionMode     `json:"execution_mode"`           // Execution mode
	Model         string            `json:"model,omitempty"`          // Model to use
	RepoURL       string            `json:"repo_url,omitempty"`       // Git repository URL
	Branch        string            `json:"branch,omitempty"`         // Git branch
	Worktree      string            `json:"worktree,omitempty"`       // Git worktree path
	ParentID      *string           `json:"parent_id,omitempty"`      // ticket.parent (for subtasks)
	Priority      int               `json:"priority"`                 // ticket.priority
	Labels        []string          `json:"labels"`                   // ticket tags
	Secrets       *EncryptedPayload `json:"secrets,omitempty"`        // age-encrypted secrets
	CreatedAt     time.Time         `json:"created_at"`               // Creation timestamp
	StartedAt     *time.Time        `json:"started_at,omitempty"`     // When execution started
	CompletedAt   *time.Time        `json:"completed_at,omitempty"`   // When execution completed
	Result        string            `json:"result,omitempty"`         // Result summary
	ErrorMessage  string            `json:"error_message,omitempty"`  // Error if failed
}

// TaskFilter defines criteria for filtering tasks.
type TaskFilter struct {
	Status    []TaskStatus `json:"status,omitempty"`
	AgentType string       `json:"agent_type,omitempty"`
	ParentID  *string      `json:"parent_id,omitempty"`
	Labels    []string     `json:"labels,omitempty"`
	Limit     int          `json:"limit,omitempty"`
	Offset    int          `json:"offset,omitempty"`
}

// TaskUpdate contains fields that can be updated on a task.
type TaskUpdate struct {
	Title         *string            `json:"title,omitempty"`
	Description   *string            `json:"description,omitempty"`
	Status        *TaskStatus        `json:"status,omitempty"`
	AgentType     *string            `json:"agent_type,omitempty"`
	ExecutionMode *ExecutionMode     `json:"execution_mode,omitempty"`
	Model         *string            `json:"model,omitempty"`
	Priority      *int               `json:"priority,omitempty"`
	Labels        []string           `json:"labels,omitempty"`
	Secrets       *EncryptedPayload  `json:"secrets,omitempty"`
	StartedAt     *time.Time         `json:"started_at,omitempty"`
	CompletedAt   *time.Time         `json:"completed_at,omitempty"`
	Result        *string            `json:"result,omitempty"`
	ErrorMessage  *string            `json:"error_message,omitempty"`
}

// TaskProgress represents progress update from an agent.
type TaskProgress struct {
	TaskID          string `json:"task_id"`
	Message         string `json:"message"`
	PercentComplete int    `json:"percent_complete"` // 0-100
}

// TaskArtifact represents an output file from task execution.
type TaskArtifact struct {
	TaskID    string    `json:"task_id"`
	Name      string    `json:"name"`
	Size      int64     `json:"size"`
	MimeType  string    `json:"mime_type,omitempty"`
	CreatedAt time.Time `json:"created_at"`
}
