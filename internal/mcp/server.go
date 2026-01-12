// Package mcp provides the MCP (Model Context Protocol) server implementation.
package mcp

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"sync"

	"github.com/roea-ai/roea/internal/core/task"
	"github.com/roea-ai/roea/internal/crypto"
	"github.com/roea-ai/roea/internal/fossil"
	"github.com/roea-ai/roea/pkg/types"
)

// Server implements the MCP protocol for agent communication.
type Server struct {
	taskManager    *task.Manager
	artifactStore  *fossil.ArtifactStore
	payloadService *crypto.PayloadService

	// Tool handlers
	toolHandlers map[string]ToolHandler

	// Active task sessions
	sessionsMu sync.RWMutex
	sessions   map[string]*Session
}

// Session represents an active MCP session for a task.
type Session struct {
	TaskID     string
	AgentID    string
	PublicKey  string // Agent's public key for encrypted responses
}

// ToolHandler is a function that handles an MCP tool call.
type ToolHandler func(session *Session, params map[string]any) (any, error)

// NewServer creates a new MCP Server.
func NewServer(
	taskManager *task.Manager,
	artifactStore *fossil.ArtifactStore,
	payloadService *crypto.PayloadService,
) *Server {
	s := &Server{
		taskManager:    taskManager,
		artifactStore:  artifactStore,
		payloadService: payloadService,
		toolHandlers:   make(map[string]ToolHandler),
		sessions:       make(map[string]*Session),
	}

	s.registerTools()
	return s
}

// registerTools sets up the MCP tool handlers.
func (s *Server) registerTools() {
	s.toolHandlers["roea_report_progress"] = s.handleReportProgress
	s.toolHandlers["roea_complete_task"] = s.handleCompleteTask
	s.toolHandlers["roea_fail_task"] = s.handleFailTask
	s.toolHandlers["roea_spawn_subtask"] = s.handleSpawnSubtask
	s.toolHandlers["roea_get_secrets"] = s.handleGetSecrets
	s.toolHandlers["roea_store_artifact"] = s.handleStoreArtifact
}

// CreateSession creates a new MCP session for a task.
func (s *Server) CreateSession(taskID string, agentID string, publicKey string) string {
	s.sessionsMu.Lock()
	defer s.sessionsMu.Unlock()

	sessionID := fmt.Sprintf("%s-%s", taskID[:8], agentID[:8])
	s.sessions[sessionID] = &Session{
		TaskID:    taskID,
		AgentID:   agentID,
		PublicKey: publicKey,
	}

	return sessionID
}

// GetSession retrieves a session by ID.
func (s *Server) GetSession(sessionID string) *Session {
	s.sessionsMu.RLock()
	defer s.sessionsMu.RUnlock()

	return s.sessions[sessionID]
}

// CloseSession removes a session.
func (s *Server) CloseSession(sessionID string) {
	s.sessionsMu.Lock()
	defer s.sessionsMu.Unlock()

	delete(s.sessions, sessionID)
}

// HandleToolCall processes an MCP tool call.
func (s *Server) HandleToolCall(sessionID string, toolName string, params map[string]any) (any, error) {
	session := s.GetSession(sessionID)
	if session == nil {
		return nil, fmt.Errorf("session not found: %s", sessionID)
	}

	handler, ok := s.toolHandlers[toolName]
	if !ok {
		return nil, fmt.Errorf("unknown tool: %s", toolName)
	}

	return handler(session, params)
}

// handleReportProgress handles the roea_report_progress tool.
func (s *Server) handleReportProgress(session *Session, params map[string]any) (any, error) {
	message, _ := params["message"].(string)
	percentComplete, _ := params["percent_complete"].(float64)

	progress := &types.TaskProgress{
		TaskID:          session.TaskID,
		Message:         message,
		PercentComplete: int(percentComplete),
	}

	s.taskManager.ReportProgress(progress)

	return map[string]any{
		"status": "ok",
	}, nil
}

// handleCompleteTask handles the roea_complete_task tool.
func (s *Server) handleCompleteTask(session *Session, params map[string]any) (any, error) {
	resultSummary, _ := params["result_summary"].(string)
	artifactsRaw, _ := params["artifacts"].([]any)

	var artifacts []string
	for _, a := range artifactsRaw {
		if s, ok := a.(string); ok {
			artifacts = append(artifacts, s)
		}
	}

	if err := s.taskManager.CompleteTask(session.TaskID, resultSummary, artifacts); err != nil {
		return nil, fmt.Errorf("failed to complete task: %w", err)
	}

	return map[string]any{
		"status": "completed",
	}, nil
}

// handleFailTask handles the roea_fail_task tool.
func (s *Server) handleFailTask(session *Session, params map[string]any) (any, error) {
	errorMsg, _ := params["error"].(string)
	// recoverable, _ := params["recoverable"].(bool)

	if err := s.taskManager.FailTask(session.TaskID, errorMsg); err != nil {
		return nil, fmt.Errorf("failed to fail task: %w", err)
	}

	return map[string]any{
		"status": "failed",
	}, nil
}

// handleSpawnSubtask handles the roea_spawn_subtask tool.
func (s *Server) handleSpawnSubtask(session *Session, params map[string]any) (any, error) {
	title, _ := params["title"].(string)
	description, _ := params["description"].(string)
	agentType, _ := params["agent_type"].(string)

	if agentType == "" {
		// Inherit from parent task
		parentTask, err := s.taskManager.GetTask(session.TaskID)
		if err == nil && parentTask != nil {
			agentType = parentTask.AgentType
		}
	}

	subtask := &types.Task{
		Title:       title,
		Description: description,
		AgentType:   agentType,
		Status:      types.TaskPending,
	}

	if err := s.taskManager.CreateSubtask(session.TaskID, subtask); err != nil {
		return nil, fmt.Errorf("failed to create subtask: %w", err)
	}

	return &types.SpawnSubtaskResult{
		TaskID: subtask.ID,
	}, nil
}

// handleGetSecrets handles the roea_get_secrets tool.
func (s *Server) handleGetSecrets(session *Session, params map[string]any) (any, error) {
	taskID, _ := params["task_id"].(string)
	if taskID == "" {
		taskID = session.TaskID
	}

	taskObj, err := s.taskManager.GetTask(taskID)
	if err != nil {
		return nil, fmt.Errorf("failed to get task: %w", err)
	}
	if taskObj == nil {
		return nil, fmt.Errorf("task not found: %s", taskID)
	}

	return &types.GetSecretsResult{
		EncryptedPayload: taskObj.Secrets,
	}, nil
}

// handleStoreArtifact handles the roea_store_artifact tool.
func (s *Server) handleStoreArtifact(session *Session, params map[string]any) (any, error) {
	taskID, _ := params["task_id"].(string)
	if taskID == "" {
		taskID = session.TaskID
	}
	filename, _ := params["filename"].(string)
	content, _ := params["content"].(string) // Base64

	if err := s.artifactStore.StoreArtifactBase64(taskID, filename, content); err != nil {
		return nil, fmt.Errorf("failed to store artifact: %w", err)
	}

	return map[string]any{
		"status": "stored",
	}, nil
}

// GetToolDefinitions returns the list of available MCP tools.
func (s *Server) GetToolDefinitions() []*types.MCPToolDefinition {
	return []*types.MCPToolDefinition{
		{
			Name:        "roea_report_progress",
			Description: "Report progress on current task",
			Parameters: map[string]types.MCPParameterDef{
				"message": {
					Type:        "string",
					Description: "Progress message",
					Required:    true,
				},
				"percent_complete": {
					Type:        "number",
					Description: "Percent complete (0-100)",
					Required:    false,
				},
			},
		},
		{
			Name:        "roea_complete_task",
			Description: "Mark current task as completed",
			Parameters: map[string]types.MCPParameterDef{
				"result_summary": {
					Type:        "string",
					Description: "Summary of work completed",
					Required:    true,
				},
				"artifacts": {
					Type:        "array",
					Description: "List of artifact file paths to store",
					Required:    false,
				},
			},
		},
		{
			Name:        "roea_fail_task",
			Description: "Mark current task as failed",
			Parameters: map[string]types.MCPParameterDef{
				"error": {
					Type:        "string",
					Description: "Error message describing the failure",
					Required:    true,
				},
				"recoverable": {
					Type:        "boolean",
					Description: "Whether the failure is recoverable",
					Required:    false,
				},
			},
		},
		{
			Name:        "roea_spawn_subtask",
			Description: "Create a subtask (new Fossil ticket)",
			Parameters: map[string]types.MCPParameterDef{
				"title": {
					Type:        "string",
					Description: "Subtask title",
					Required:    true,
				},
				"description": {
					Type:        "string",
					Description: "Subtask description",
					Required:    true,
				},
				"agent_type": {
					Type:        "string",
					Description: "Agent type for subtask (optional)",
					Required:    false,
				},
			},
		},
		{
			Name:        "roea_get_secrets",
			Description: "Get age-encrypted secrets for task",
			Parameters: map[string]types.MCPParameterDef{
				"task_id": {
					Type:        "string",
					Description: "Task ID (defaults to current task)",
					Required:    false,
				},
			},
		},
		{
			Name:        "roea_store_artifact",
			Description: "Store output file in Fossil",
			Parameters: map[string]types.MCPParameterDef{
				"task_id": {
					Type:        "string",
					Description: "Task ID (defaults to current task)",
					Required:    false,
				},
				"filename": {
					Type:        "string",
					Description: "Artifact filename",
					Required:    true,
				},
				"content": {
					Type:        "string",
					Description: "Base64-encoded file content",
					Required:    true,
				},
			},
		},
	}
}

// ServeHTTP implements http.Handler for the MCP server.
func (s *Server) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	// Read request body
	body, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, "Failed to read body", http.StatusBadRequest)
		return
	}
	defer r.Body.Close()

	// Parse MCP request
	var req types.MCPRequest
	if err := json.Unmarshal(body, &req); err != nil {
		http.Error(w, "Invalid JSON", http.StatusBadRequest)
		return
	}

	// Get session ID from query params or header
	sessionID := r.URL.Query().Get("session")
	if sessionID == "" {
		sessionID = r.Header.Get("X-Roea-Session")
	}
	if sessionID == "" {
		http.Error(w, "Session ID required", http.StatusBadRequest)
		return
	}

	// Handle special methods
	switch req.Method {
	case "tools/list":
		s.respondJSON(w, map[string]any{
			"tools": s.GetToolDefinitions(),
		})
		return
	case "tools/call":
		toolName, _ := req.Params["name"].(string)
		arguments, _ := req.Params["arguments"].(map[string]any)

		result, err := s.HandleToolCall(sessionID, toolName, arguments)
		if err != nil {
			s.respondJSON(w, &types.MCPResponse{
				Error: err.Error(),
			})
			return
		}

		s.respondJSON(w, &types.MCPResponse{
			Result: result,
		})
		return
	default:
		http.Error(w, "Unknown method", http.StatusBadRequest)
	}
}

func (s *Server) respondJSON(w http.ResponseWriter, data any) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(data)
}
