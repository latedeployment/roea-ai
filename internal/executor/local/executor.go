// Package local provides a local subprocess executor for agents.
package local

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"sync"
	"syscall"
	"time"

	"github.com/roea-ai/roea/internal/core/execution"
	"github.com/roea-ai/roea/pkg/types"
)

// Executor runs agents as local subprocesses.
type Executor struct {
	config *types.LocalExecutorConfig

	// Running processes
	processesMu sync.RWMutex
	processes   map[string]*Process

	// Semaphore for max concurrent
	semaphore chan struct{}
}

// Process represents a running agent process.
type Process struct {
	InstanceID string
	TaskID     string
	Cmd        *exec.Cmd
	StartedAt  time.Time
	Output     *bytes.Buffer
	Cancel     context.CancelFunc
}

// NewExecutor creates a new local Executor.
func NewExecutor(config *types.LocalExecutorConfig) *Executor {
	if config.MaxConcurrent <= 0 {
		config.MaxConcurrent = 4
	}

	return &Executor{
		config:    config,
		processes: make(map[string]*Process),
		semaphore: make(chan struct{}, config.MaxConcurrent),
	}
}

// Name returns the executor name.
func (e *Executor) Name() string {
	return "local"
}

// CanExecute checks if this executor can handle the task.
func (e *Executor) CanExecute(task *types.Task, agent *types.AgentDefinition) bool {
	if !e.config.Enabled {
		return false
	}

	// Local executor can handle any runtime
	return true
}

// Execute runs the agent for the task.
func (e *Executor) Execute(ctx context.Context, req *execution.ExecutionRequest) (*execution.ExecutionResult, error) {
	// Acquire semaphore slot
	select {
	case e.semaphore <- struct{}{}:
		defer func() { <-e.semaphore }()
	case <-ctx.Done():
		return nil, ctx.Err()
	}

	// Determine runtime command
	var cmd *exec.Cmd
	switch types.AgentRuntime(req.Agent.BaseRuntime) {
	case types.RuntimeClaudeCode:
		cmd = e.buildClaudeCodeCommand(ctx, req)
	case types.RuntimeOpenCode:
		cmd = e.buildOpenCodeCommand(ctx, req)
	case types.RuntimeCodex:
		cmd = e.buildCodexCommand(ctx, req)
	default:
		// Default to claude-code
		cmd = e.buildClaudeCodeCommand(ctx, req)
	}

	// Set working directory
	if req.Worktree != "" {
		cmd.Dir = req.Worktree
	}

	// Capture output
	var output bytes.Buffer
	cmd.Stdout = &output
	cmd.Stderr = &output

	// Create cancellable context
	cmdCtx, cancel := context.WithCancel(ctx)
	cmd = exec.CommandContext(cmdCtx, cmd.Path, cmd.Args[1:]...)
	if req.Worktree != "" {
		cmd.Dir = req.Worktree
	}
	cmd.Stdout = &output
	cmd.Stderr = &output
	cmd.Env = e.buildEnvironment(req)

	// Register process
	proc := &Process{
		InstanceID: req.InstanceID,
		TaskID:     req.Task.ID,
		Cmd:        cmd,
		StartedAt:  time.Now(),
		Output:     &output,
		Cancel:     cancel,
	}

	e.processesMu.Lock()
	e.processes[req.InstanceID] = proc
	e.processesMu.Unlock()

	defer func() {
		e.processesMu.Lock()
		delete(e.processes, req.InstanceID)
		e.processesMu.Unlock()
	}()

	// Start the process
	if err := cmd.Start(); err != nil {
		return &execution.ExecutionResult{
			InstanceID:   req.InstanceID,
			Success:      false,
			ErrorMessage: fmt.Sprintf("failed to start process: %v", err),
		}, nil
	}

	// Wait for completion
	err := cmd.Wait()

	result := &execution.ExecutionResult{
		InstanceID: req.InstanceID,
		Output:     output.String(),
	}

	if err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			result.ExitCode = exitErr.ExitCode()
		}
		result.Success = false
		result.ErrorMessage = err.Error()
	} else {
		result.Success = true
		result.ExitCode = 0
	}

	return result, nil
}

// Stop stops a running execution.
func (e *Executor) Stop(instanceID string) error {
	e.processesMu.RLock()
	proc, ok := e.processes[instanceID]
	e.processesMu.RUnlock()

	if !ok {
		return fmt.Errorf("process not found: %s", instanceID)
	}

	// Cancel context
	proc.Cancel()

	// Send SIGTERM
	if proc.Cmd.Process != nil {
		proc.Cmd.Process.Signal(syscall.SIGTERM)

		// Give it a moment to gracefully shutdown
		time.Sleep(2 * time.Second)

		// Force kill if still running
		if e.IsRunning(instanceID) {
			proc.Cmd.Process.Kill()
		}
	}

	return nil
}

// IsRunning checks if an instance is still running.
func (e *Executor) IsRunning(instanceID string) bool {
	e.processesMu.RLock()
	proc, ok := e.processes[instanceID]
	e.processesMu.RUnlock()

	if !ok {
		return false
	}

	// Check if process is still alive
	if proc.Cmd.Process != nil {
		err := proc.Cmd.Process.Signal(syscall.Signal(0))
		return err == nil
	}

	return false
}

// buildClaudeCodeCommand builds the command for Claude Code runtime.
func (e *Executor) buildClaudeCodeCommand(ctx context.Context, req *execution.ExecutionRequest) *exec.Cmd {
	args := []string{
		"--print",
		"--output-format", "json",
	}

	// Add system prompt if specified
	if req.Agent.SystemPrompt != "" {
		args = append(args, "--system-prompt", req.Agent.SystemPrompt)
	}

	// Add the task description as the prompt
	args = append(args, "--prompt", req.Task.Description)

	return exec.Command("claude", args...)
}

// buildOpenCodeCommand builds the command for OpenCode runtime.
func (e *Executor) buildOpenCodeCommand(ctx context.Context, req *execution.ExecutionRequest) *exec.Cmd {
	args := []string{
		"--non-interactive",
	}

	// Add task description
	args = append(args, req.Task.Description)

	return exec.Command("opencode", args...)
}

// buildCodexCommand builds the command for Codex runtime.
func (e *Executor) buildCodexCommand(ctx context.Context, req *execution.ExecutionRequest) *exec.Cmd {
	args := []string{
		"--quiet",
	}

	// Add task description
	args = append(args, req.Task.Description)

	return exec.Command("codex", args...)
}

// buildEnvironment builds the environment variables for the process.
func (e *Executor) buildEnvironment(req *execution.ExecutionRequest) []string {
	env := os.Environ()

	// Add Roea-specific variables
	env = append(env, fmt.Sprintf("ROEA_TASK_ID=%s", req.Task.ID))
	env = append(env, fmt.Sprintf("ROEA_INSTANCE_ID=%s", req.InstanceID))

	if req.MCPServerURL != "" {
		env = append(env, fmt.Sprintf("ROEA_MCP_URL=%s", req.MCPServerURL))
	}

	// Add model if specified
	if req.Task.Model != "" {
		env = append(env, fmt.Sprintf("ANTHROPIC_MODEL=%s", req.Task.Model))
	} else if req.Agent.DefaultModel != "" {
		env = append(env, fmt.Sprintf("ANTHROPIC_MODEL=%s", req.Agent.DefaultModel))
	}

	// Add secrets as environment variables if available
	if req.Secrets != nil {
		for key, value := range req.Secrets.APIKeys {
			env = append(env, fmt.Sprintf("%s=%s", key, value))
		}
		for key, value := range req.Secrets.Tokens {
			env = append(env, fmt.Sprintf("%s=%s", key, value))
		}
	}

	return env
}

// GetProcessInfo returns information about a running process.
func (e *Executor) GetProcessInfo(instanceID string) (map[string]any, error) {
	e.processesMu.RLock()
	proc, ok := e.processes[instanceID]
	e.processesMu.RUnlock()

	if !ok {
		return nil, fmt.Errorf("process not found: %s", instanceID)
	}

	info := map[string]any{
		"instance_id": proc.InstanceID,
		"task_id":     proc.TaskID,
		"started_at":  proc.StartedAt.Format(time.RFC3339),
		"running":     e.IsRunning(instanceID),
	}

	if proc.Cmd.Process != nil {
		info["pid"] = proc.Cmd.Process.Pid
	}

	return info, nil
}

// ListProcesses returns all running processes.
func (e *Executor) ListProcesses() []map[string]any {
	e.processesMu.RLock()
	defer e.processesMu.RUnlock()

	var procs []map[string]any
	for id := range e.processes {
		if info, err := e.GetProcessInfo(id); err == nil {
			procs = append(procs, info)
		}
	}

	return procs
}

// GetOutput returns the current output of a running process.
func (e *Executor) GetOutput(instanceID string) (string, error) {
	e.processesMu.RLock()
	proc, ok := e.processes[instanceID]
	e.processesMu.RUnlock()

	if !ok {
		return "", fmt.Errorf("process not found: %s", instanceID)
	}

	return proc.Output.String(), nil
}

// StreamOutput returns output as it's generated (for real-time streaming).
func (e *Executor) StreamOutput(instanceID string, callback func(string)) error {
	e.processesMu.RLock()
	proc, ok := e.processes[instanceID]
	e.processesMu.RUnlock()

	if !ok {
		return fmt.Errorf("process not found: %s", instanceID)
	}

	lastLen := 0
	for e.IsRunning(instanceID) {
		output := proc.Output.String()
		if len(output) > lastLen {
			callback(output[lastLen:])
			lastLen = len(output)
		}
		time.Sleep(100 * time.Millisecond)
	}

	// Get any remaining output
	output := proc.Output.String()
	if len(output) > lastLen {
		callback(output[lastLen:])
	}

	return nil
}

// ParseClaudeCodeOutput parses JSON output from Claude Code.
func ParseClaudeCodeOutput(output string) (map[string]any, error) {
	var result map[string]any
	if err := json.Unmarshal([]byte(output), &result); err != nil {
		return nil, err
	}
	return result, nil
}
