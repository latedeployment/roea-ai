package types

// MCPRequest represents an incoming MCP tool request.
type MCPRequest struct {
	Method string         `json:"method"`
	Params map[string]any `json:"params"`
}

// MCPResponse represents an MCP tool response.
type MCPResponse struct {
	Result any    `json:"result,omitempty"`
	Error  string `json:"error,omitempty"`
}

// ReportProgressParams for roea_report_progress tool.
type ReportProgressParams struct {
	TaskID          string `json:"task_id"`
	Message         string `json:"message"`
	PercentComplete int    `json:"percent_complete"` // 0-100
}

// CompleteTaskParams for roea_complete_task tool.
type CompleteTaskParams struct {
	TaskID        string   `json:"task_id"`
	ResultSummary string   `json:"result_summary"`
	Artifacts     []string `json:"artifacts,omitempty"` // File paths to store
}

// FailTaskParams for roea_fail_task tool.
type FailTaskParams struct {
	TaskID      string `json:"task_id"`
	Error       string `json:"error"`
	Recoverable bool   `json:"recoverable"`
}

// SpawnSubtaskParams for roea_spawn_subtask tool.
type SpawnSubtaskParams struct {
	Title       string `json:"title"`
	Description string `json:"description"`
	AgentType   string `json:"agent_type,omitempty"`
}

// SpawnSubtaskResult returned from roea_spawn_subtask.
type SpawnSubtaskResult struct {
	TaskID string `json:"task_id"`
}

// GetSecretsParams for roea_get_secrets tool.
type GetSecretsParams struct {
	TaskID string `json:"task_id"`
}

// GetSecretsResult returned from roea_get_secrets.
type GetSecretsResult struct {
	EncryptedPayload *EncryptedPayload `json:"encrypted_payload"`
}

// StoreArtifactParams for roea_store_artifact tool.
type StoreArtifactParams struct {
	TaskID   string `json:"task_id"`
	Filename string `json:"filename"`
	Content  string `json:"content"` // Base64-encoded content
}

// MCPToolDefinition describes an MCP tool.
type MCPToolDefinition struct {
	Name        string                       `json:"name"`
	Description string                       `json:"description"`
	Parameters  map[string]MCPParameterDef   `json:"parameters"`
}

// MCPParameterDef describes an MCP tool parameter.
type MCPParameterDef struct {
	Type        string `json:"type"`
	Description string `json:"description"`
	Required    bool   `json:"required,omitempty"`
}
