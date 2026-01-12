// Package execution provides the execution engine for running agents.
package execution

import (
	"context"
	"fmt"
	"sync"

	"github.com/roea-ai/roea/internal/core/agent"
	"github.com/roea-ai/roea/internal/core/task"
	"github.com/roea-ai/roea/pkg/types"
)

// Executor is the interface for execution backends.
type Executor interface {
	// Name returns the executor name.
	Name() string

	// CanExecute checks if the executor can handle this task.
	CanExecute(task *types.Task, agent *types.AgentDefinition) bool

	// Execute runs the agent for the task.
	Execute(ctx context.Context, req *ExecutionRequest) (*ExecutionResult, error)

	// Stop stops a running execution.
	Stop(instanceID string) error

	// IsRunning checks if an instance is still running.
	IsRunning(instanceID string) bool
}

// ExecutionRequest contains all data needed to execute a task.
type ExecutionRequest struct {
	Task            *types.Task
	Agent           *types.AgentDefinition
	Secrets         *types.TaskSecrets
	Worktree        string
	MCPServerURL    string
	InstanceID      string
}

// ExecutionResult contains the result of an execution.
type ExecutionResult struct {
	InstanceID   string
	Success      bool
	ExitCode     int
	Output       string
	ErrorMessage string
}

// Engine coordinates task execution across multiple backends.
type Engine struct {
	taskManager *task.Manager
	agentPool   *agent.Pool

	executors   []Executor
	executorsMu sync.RWMutex

	// Active executions
	activeExecs   map[string]*activeExecution
	activeExecsMu sync.RWMutex
}

type activeExecution struct {
	InstanceID string
	TaskID     string
	Executor   Executor
	Cancel     context.CancelFunc
}

// NewEngine creates a new execution Engine.
func NewEngine(taskManager *task.Manager, agentPool *agent.Pool) *Engine {
	return &Engine{
		taskManager: taskManager,
		agentPool:   agentPool,
		executors:   make([]Executor, 0),
		activeExecs: make(map[string]*activeExecution),
	}
}

// RegisterExecutor adds an executor backend.
func (e *Engine) RegisterExecutor(executor Executor) {
	e.executorsMu.Lock()
	defer e.executorsMu.Unlock()
	e.executors = append(e.executors, executor)
}

// Execute starts execution of a task.
func (e *Engine) Execute(ctx context.Context, taskID string) (*ExecutionResult, error) {
	// Get the task
	taskObj, secrets, err := e.taskManager.GetTaskWithSecrets(taskID)
	if err != nil {
		return nil, fmt.Errorf("failed to get task: %w", err)
	}
	if taskObj == nil {
		return nil, fmt.Errorf("task not found: %s", taskID)
	}

	// Get the agent definition
	agentDef, err := e.agentPool.GetAgentDefinition(taskObj.AgentType)
	if err != nil {
		return nil, fmt.Errorf("failed to get agent definition: %w", err)
	}

	// Find an appropriate executor
	executor := e.findExecutor(taskObj, agentDef)
	if executor == nil {
		return nil, fmt.Errorf("no executor available for task")
	}

	// Generate instance ID
	instanceID := fmt.Sprintf("%s-%s", taskID[:8], generateShortID())

	// Create execution context with cancellation
	execCtx, cancel := context.WithCancel(ctx)

	// Register active execution
	e.activeExecsMu.Lock()
	e.activeExecs[instanceID] = &activeExecution{
		InstanceID: instanceID,
		TaskID:     taskID,
		Executor:   executor,
		Cancel:     cancel,
	}
	e.activeExecsMu.Unlock()

	// Register agent instance
	e.agentPool.RegisterInstance(&types.AgentInstance{
		ID:         instanceID,
		AgentType:  taskObj.AgentType,
		TaskID:     taskID,
		ExecutorID: executor.Name(),
		Status:     "starting",
	})

	// Mark task as running
	if err := e.taskManager.AssignTask(taskID, instanceID); err != nil {
		return nil, fmt.Errorf("failed to assign task: %w", err)
	}

	// Build execution request
	req := &ExecutionRequest{
		Task:       taskObj,
		Agent:      agentDef,
		Secrets:    secrets,
		Worktree:   taskObj.Worktree,
		InstanceID: instanceID,
	}

	// Execute
	result, err := executor.Execute(execCtx, req)

	// Cleanup
	e.activeExecsMu.Lock()
	delete(e.activeExecs, instanceID)
	e.activeExecsMu.Unlock()

	e.agentPool.UnregisterInstance(instanceID)

	if err != nil {
		e.taskManager.FailTask(taskID, err.Error())
		return nil, err
	}

	// Update task status based on result
	if result.Success {
		e.taskManager.CompleteTask(taskID, result.Output, nil)
	} else {
		e.taskManager.FailTask(taskID, result.ErrorMessage)
	}

	return result, nil
}

// ExecuteAsync starts execution in the background.
func (e *Engine) ExecuteAsync(taskID string) (string, error) {
	// Get the task
	taskObj, _, err := e.taskManager.GetTaskWithSecrets(taskID)
	if err != nil {
		return "", fmt.Errorf("failed to get task: %w", err)
	}
	if taskObj == nil {
		return "", fmt.Errorf("task not found: %s", taskID)
	}

	// Get the agent definition
	agentDef, err := e.agentPool.GetAgentDefinition(taskObj.AgentType)
	if err != nil {
		return "", fmt.Errorf("failed to get agent definition: %w", err)
	}

	// Find an appropriate executor
	executor := e.findExecutor(taskObj, agentDef)
	if executor == nil {
		return "", fmt.Errorf("no executor available for task")
	}

	// Generate instance ID
	instanceID := fmt.Sprintf("%s-%s", taskID[:8], generateShortID())

	// Start background execution
	go func() {
		ctx := context.Background()
		e.Execute(ctx, taskID)
	}()

	return instanceID, nil
}

// Stop stops a running execution.
func (e *Engine) Stop(instanceID string) error {
	e.activeExecsMu.RLock()
	exec, ok := e.activeExecs[instanceID]
	e.activeExecsMu.RUnlock()

	if !ok {
		return fmt.Errorf("execution not found: %s", instanceID)
	}

	// Cancel the context
	exec.Cancel()

	// Stop via executor
	if err := exec.Executor.Stop(instanceID); err != nil {
		return fmt.Errorf("failed to stop execution: %w", err)
	}

	return nil
}

// StopTask stops execution for a task.
func (e *Engine) StopTask(taskID string) error {
	e.activeExecsMu.RLock()
	var instanceID string
	for id, exec := range e.activeExecs {
		if exec.TaskID == taskID {
			instanceID = id
			break
		}
	}
	e.activeExecsMu.RUnlock()

	if instanceID == "" {
		return fmt.Errorf("no running execution for task: %s", taskID)
	}

	return e.Stop(instanceID)
}

// GetActiveExecutions returns all active executions.
func (e *Engine) GetActiveExecutions() []string {
	e.activeExecsMu.RLock()
	defer e.activeExecsMu.RUnlock()

	ids := make([]string, 0, len(e.activeExecs))
	for id := range e.activeExecs {
		ids = append(ids, id)
	}
	return ids
}

// IsExecuting checks if a task is currently being executed.
func (e *Engine) IsExecuting(taskID string) bool {
	e.activeExecsMu.RLock()
	defer e.activeExecsMu.RUnlock()

	for _, exec := range e.activeExecs {
		if exec.TaskID == taskID {
			return true
		}
	}
	return false
}

// findExecutor finds an appropriate executor for the task.
func (e *Engine) findExecutor(taskObj *types.Task, agentDef *types.AgentDefinition) Executor {
	e.executorsMu.RLock()
	defer e.executorsMu.RUnlock()

	for _, exec := range e.executors {
		if exec.CanExecute(taskObj, agentDef) {
			return exec
		}
	}
	return nil
}

// generateShortID generates a short random ID.
func generateShortID() string {
	const chars = "abcdefghijklmnopqrstuvwxyz0123456789"
	result := make([]byte, 6)
	for i := range result {
		result[i] = chars[i%len(chars)]
	}
	return string(result)
}
