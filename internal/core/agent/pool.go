// Package agent provides agent management functionality.
package agent

import (
	"fmt"
	"sync"
	"time"

	"github.com/roea-ai/roea/internal/fossil"
	"github.com/roea-ai/roea/pkg/types"
)

// Pool manages agent definitions and running instances.
type Pool struct {
	wikiStore *fossil.WikiStore

	// Running instances
	instancesMu sync.RWMutex
	instances   map[string]*types.AgentInstance

	// Default agent definitions (built-in)
	builtinAgents map[string]*types.AgentDefinition
}

// NewPool creates a new agent Pool.
func NewPool(wikiStore *fossil.WikiStore) *Pool {
	p := &Pool{
		wikiStore:     wikiStore,
		instances:     make(map[string]*types.AgentInstance),
		builtinAgents: make(map[string]*types.AgentDefinition),
	}

	// Register built-in agents
	p.registerBuiltinAgents()

	return p
}

// registerBuiltinAgents sets up default agent definitions.
func (p *Pool) registerBuiltinAgents() {
	p.builtinAgents["general-coder"] = &types.AgentDefinition{
		ID:          "general-coder",
		Name:        "General Coder",
		Description: "General-purpose coding agent for various programming tasks",
		BaseRuntime: string(types.RuntimeClaudeCode),
		SystemPrompt: `You are a skilled software developer working on a coding task.
Follow best practices, write clean code, and ensure your changes are well-tested.
When done, use the roea_complete_task tool to mark the task as finished.`,
		MCPServers:   []string{"roea", "filesystem"},
		DefaultModel: "claude-sonnet-4-20250514",
		ResourceLimits: &types.ResourceLimits{
			MaxTurns:       50,
			TimeoutMinutes: 30,
			MaxCostUSD:     5.00,
		},
	}

	p.builtinAgents["bug-fixer"] = &types.AgentDefinition{
		ID:          "bug-fixer",
		Name:        "Bug Fixer",
		Description: "Specialized agent for debugging and fixing issues",
		BaseRuntime: string(types.RuntimeClaudeCode),
		SystemPrompt: `You are an expert debugger. Analyze the bug report,
reproduce the issue, identify the root cause, and implement a fix.
Write tests to prevent regression. Use roea_complete_task when done.`,
		MCPServers:   []string{"roea", "filesystem", "github"},
		DefaultModel: "claude-sonnet-4-20250514",
		ResourceLimits: &types.ResourceLimits{
			MaxTurns:       100,
			TimeoutMinutes: 60,
			MaxCostUSD:     10.00,
		},
	}

	p.builtinAgents["reviewer"] = &types.AgentDefinition{
		ID:          "reviewer",
		Name:        "Code Reviewer",
		Description: "Reviews code changes and provides feedback",
		BaseRuntime: string(types.RuntimeClaudeCode),
		SystemPrompt: `You are a thorough code reviewer. Review the changes for:
- Code quality and style
- Potential bugs
- Security issues
- Performance concerns
- Test coverage
Provide constructive feedback and suggestions.`,
		MCPServers:   []string{"roea", "filesystem", "github"},
		DefaultModel: "claude-sonnet-4-20250514",
		ResourceLimits: &types.ResourceLimits{
			MaxTurns:       30,
			TimeoutMinutes: 15,
			MaxCostUSD:     3.00,
		},
	}

	p.builtinAgents["docs-writer"] = &types.AgentDefinition{
		ID:          "docs-writer",
		Name:        "Documentation Writer",
		Description: "Creates and updates documentation",
		BaseRuntime: string(types.RuntimeClaudeCode),
		SystemPrompt: `You are a technical writer. Create clear, accurate documentation.
Include examples, explain concepts thoroughly, and maintain consistent style.`,
		MCPServers:   []string{"roea", "filesystem"},
		DefaultModel: "claude-sonnet-4-20250514",
		ResourceLimits: &types.ResourceLimits{
			MaxTurns:       40,
			TimeoutMinutes: 20,
			MaxCostUSD:     4.00,
		},
	}

	p.builtinAgents["test-writer"] = &types.AgentDefinition{
		ID:          "test-writer",
		Name:        "Test Writer",
		Description: "Creates comprehensive test suites",
		BaseRuntime: string(types.RuntimeClaudeCode),
		SystemPrompt: `You are a QA engineer writing tests. Create comprehensive test suites
covering edge cases, error conditions, and happy paths. Ensure high coverage.`,
		MCPServers:   []string{"roea", "filesystem"},
		DefaultModel: "claude-sonnet-4-20250514",
		ResourceLimits: &types.ResourceLimits{
			MaxTurns:       60,
			TimeoutMinutes: 45,
			MaxCostUSD:     6.00,
		},
	}
}

// GetAgentDefinition retrieves an agent definition by ID.
func (p *Pool) GetAgentDefinition(id string) (*types.AgentDefinition, error) {
	// Check wiki store first
	if p.wikiStore != nil {
		agent, err := p.wikiStore.GetAgentDefinition(id)
		if err != nil {
			return nil, err
		}
		if agent != nil {
			return agent, nil
		}
	}

	// Fall back to built-in
	if agent, ok := p.builtinAgents[id]; ok {
		return agent, nil
	}

	return nil, fmt.Errorf("agent definition not found: %s", id)
}

// ListAgentDefinitions returns all available agent definitions.
func (p *Pool) ListAgentDefinitions() ([]*types.AgentDefinition, error) {
	agents := make([]*types.AgentDefinition, 0)

	// Add built-in agents
	for _, agent := range p.builtinAgents {
		agents = append(agents, agent)
	}

	// Add custom agents from wiki
	if p.wikiStore != nil {
		customAgents, err := p.wikiStore.ListAgentDefinitions()
		if err != nil {
			return nil, err
		}
		agents = append(agents, customAgents...)
	}

	return agents, nil
}

// SaveAgentDefinition saves a custom agent definition.
func (p *Pool) SaveAgentDefinition(agent *types.AgentDefinition) error {
	if p.wikiStore == nil {
		return fmt.Errorf("wiki store not configured")
	}
	return p.wikiStore.SaveAgentDefinition(agent)
}

// DeleteAgentDefinition removes a custom agent definition.
func (p *Pool) DeleteAgentDefinition(id string) error {
	// Cannot delete built-in agents
	if _, ok := p.builtinAgents[id]; ok {
		return fmt.Errorf("cannot delete built-in agent: %s", id)
	}

	if p.wikiStore == nil {
		return fmt.Errorf("wiki store not configured")
	}
	return p.wikiStore.DeleteAgentDefinition(id)
}

// RegisterInstance registers a running agent instance.
func (p *Pool) RegisterInstance(instance *types.AgentInstance) {
	p.instancesMu.Lock()
	defer p.instancesMu.Unlock()

	instance.StartedAt = time.Now().Format(time.RFC3339)
	instance.Status = "running"
	p.instances[instance.ID] = instance
}

// UnregisterInstance removes a running agent instance.
func (p *Pool) UnregisterInstance(instanceID string) {
	p.instancesMu.Lock()
	defer p.instancesMu.Unlock()

	delete(p.instances, instanceID)
}

// GetInstance retrieves a running instance by ID.
func (p *Pool) GetInstance(instanceID string) *types.AgentInstance {
	p.instancesMu.RLock()
	defer p.instancesMu.RUnlock()

	return p.instances[instanceID]
}

// ListInstances returns all running agent instances.
func (p *Pool) ListInstances() []*types.AgentInstance {
	p.instancesMu.RLock()
	defer p.instancesMu.RUnlock()

	instances := make([]*types.AgentInstance, 0, len(p.instances))
	for _, inst := range p.instances {
		instances = append(instances, inst)
	}
	return instances
}

// GetInstanceByTask finds an instance running a specific task.
func (p *Pool) GetInstanceByTask(taskID string) *types.AgentInstance {
	p.instancesMu.RLock()
	defer p.instancesMu.RUnlock()

	for _, inst := range p.instances {
		if inst.TaskID == taskID {
			return inst
		}
	}
	return nil
}

// UpdateInstanceStatus updates the status of a running instance.
func (p *Pool) UpdateInstanceStatus(instanceID string, status string) {
	p.instancesMu.Lock()
	defer p.instancesMu.Unlock()

	if inst, ok := p.instances[instanceID]; ok {
		inst.Status = status
	}
}

// GetInstanceCount returns the number of running instances.
func (p *Pool) GetInstanceCount() int {
	p.instancesMu.RLock()
	defer p.instancesMu.RUnlock()

	return len(p.instances)
}

// GetInstanceCountByAgent returns instance count by agent type.
func (p *Pool) GetInstanceCountByAgent(agentType string) int {
	p.instancesMu.RLock()
	defer p.instancesMu.RUnlock()

	count := 0
	for _, inst := range p.instances {
		if inst.AgentType == agentType {
			count++
		}
	}
	return count
}
