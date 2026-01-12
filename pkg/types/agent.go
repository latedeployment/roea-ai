package types

// AgentDefinition describes an agent configuration.
// Stored in Fossil Wiki as YAML.
type AgentDefinition struct {
	ID             string          `json:"id" yaml:"id"`
	Name           string          `json:"name" yaml:"name"`
	Description    string          `json:"description" yaml:"description"`
	BaseRuntime    string          `json:"base_runtime" yaml:"base_runtime"`       // e.g., "claude-code", "opencode", "codex"
	SystemPrompt   string          `json:"system_prompt" yaml:"system_prompt"`
	MCPServers     []string        `json:"mcp_servers" yaml:"mcp_servers"`         // e.g., ["roea", "filesystem", "github"]
	DefaultModel   string          `json:"default_model" yaml:"default_model"`
	ResourceLimits *ResourceLimits `json:"resource_limits" yaml:"resource_limits"`
}

// ResourceLimits defines constraints for agent execution.
type ResourceLimits struct {
	MaxTurns       int     `json:"max_turns" yaml:"max_turns"`
	TimeoutMinutes int     `json:"timeout_minutes" yaml:"timeout_minutes"`
	MaxCostUSD     float64 `json:"max_cost_usd" yaml:"max_cost_usd"`
}

// AgentInstance represents a running agent process.
type AgentInstance struct {
	ID         string `json:"id"`
	AgentType  string `json:"agent_type"`
	TaskID     string `json:"task_id"`
	ExecutorID string `json:"executor_id"`
	PID        int    `json:"pid,omitempty"`         // For local executor
	PodName    string `json:"pod_name,omitempty"`    // For K8s executor
	VMID       string `json:"vm_id,omitempty"`       // For VM executor
	Status     string `json:"status"`                // "starting", "running", "stopping", "stopped"
	StartedAt  string `json:"started_at"`
}

// AgentRuntime represents a supported agent runtime.
type AgentRuntime string

const (
	RuntimeClaudeCode AgentRuntime = "claude-code"
	RuntimeOpenCode   AgentRuntime = "opencode"
	RuntimeCodex      AgentRuntime = "codex"
)
