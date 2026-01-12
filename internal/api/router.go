// Package api provides the REST API for Roea.
package api

import (
	"encoding/json"
	"net/http"
	"sync"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/gorilla/websocket"

	"github.com/roea-ai/roea/internal/api/handlers"
	"github.com/roea-ai/roea/internal/core/agent"
	"github.com/roea-ai/roea/internal/core/execution"
	"github.com/roea-ai/roea/internal/core/git"
	"github.com/roea-ai/roea/internal/core/process"
	"github.com/roea-ai/roea/internal/core/task"
	"github.com/roea-ai/roea/internal/mcp"
	"github.com/roea-ai/roea/pkg/types"
)

// Router holds all API dependencies and routes.
type Router struct {
	engine          *gin.Engine
	taskManager     *task.Manager
	agentPool       *agent.Pool
	executionEngine *execution.Engine
	gitManager      *git.Manager
	mcpServer       *mcp.Server
	processTracker  *process.Tracker

	// WebSocket upgrader
	upgrader websocket.Upgrader

	// WebSocket clients
	wsClientsMu sync.RWMutex
	wsClients   map[*websocket.Conn]bool
}

// NewRouter creates a new API router.
func NewRouter(
	taskManager *task.Manager,
	agentPool *agent.Pool,
	executionEngine *execution.Engine,
	gitManager *git.Manager,
	mcpServer *mcp.Server,
	processTracker *process.Tracker,
) *Router {
	r := &Router{
		engine:          gin.Default(),
		taskManager:     taskManager,
		agentPool:       agentPool,
		executionEngine: executionEngine,
		gitManager:      gitManager,
		mcpServer:       mcpServer,
		processTracker:  processTracker,
		upgrader: websocket.Upgrader{
			CheckOrigin: func(r *http.Request) bool {
				return true // Allow all origins for development
			},
		},
		wsClients: make(map[*websocket.Conn]bool),
	}

	r.setupRoutes()

	// Start broadcasting process events if tracker is available
	if processTracker != nil {
		go r.broadcastProcessEvents()
	}

	return r
}

// setupRoutes configures all API routes.
func (r *Router) setupRoutes() {
	// Health check
	r.engine.GET("/health", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	// API v1 group
	v1 := r.engine.Group("/api/v1")
	{
		// Tasks
		tasks := v1.Group("/tasks")
		{
			tasks.GET("", r.listTasks)
			tasks.POST("", r.createTask)
			tasks.GET("/next", r.getNextTask)
			tasks.GET("/stats", r.getTaskStats)
			tasks.GET("/:id", r.getTask)
			tasks.PUT("/:id", r.updateTask)
			tasks.DELETE("/:id", r.deleteTask)
			tasks.POST("/:id/execute", r.executeTask)
			tasks.POST("/:id/stop", r.stopTask)
			tasks.GET("/:id/progress", r.getTaskProgress)
			tasks.GET("/:id/artifacts", r.listTaskArtifacts)
			tasks.GET("/:id/artifacts/:name", r.getTaskArtifact)
		}

		// Agents
		agents := v1.Group("/agents")
		{
			agents.GET("", r.listAgents)
			agents.POST("", r.createAgent)
			agents.GET("/:id", r.getAgent)
			agents.PUT("/:id", r.updateAgent)
			agents.DELETE("/:id", r.deleteAgent)
			agents.GET("/instances", r.listAgentInstances)
		}

		// Git
		gitRoutes := v1.Group("/git")
		{
			gitRoutes.POST("/clone", r.cloneRepo)
			gitRoutes.GET("/worktrees", r.listWorktrees)
			gitRoutes.POST("/worktrees", r.createWorktree)
			gitRoutes.DELETE("/worktrees/:taskId", r.removeWorktree)
		}

		// MCP
		v1.Any("/mcp", r.handleMCP)

		// Processes
		processes := v1.Group("/processes")
		{
			processes.GET("", r.listProcesses)
			processes.GET("/active", r.getActiveProcesses)
			processes.GET("/graph", r.getProcessGraph)
			processes.GET("/stats", r.getProcessStats)
			processes.GET("/:id", r.getProcess)
			processes.GET("/:id/tree", r.getProcessTree)
			processes.DELETE("/:id", r.terminateProcess)
			processes.GET("/task/:taskId", r.getProcessesByTask)
			processes.GET("/instance/:instanceId", r.getProcessesByInstance)
		}
	}

	// WebSocket for real-time updates
	r.engine.GET("/ws", r.handleWebSocket)
}

// Handler returns the HTTP handler.
func (r *Router) Handler() http.Handler {
	return r.engine
}

// Task handlers

func (r *Router) listTasks(c *gin.Context) {
	h := handlers.NewTaskHandler(r.taskManager, r.executionEngine)
	h.List(c)
}

func (r *Router) createTask(c *gin.Context) {
	h := handlers.NewTaskHandler(r.taskManager, r.executionEngine)
	h.Create(c)
}

func (r *Router) getTask(c *gin.Context) {
	h := handlers.NewTaskHandler(r.taskManager, r.executionEngine)
	h.Get(c)
}

func (r *Router) updateTask(c *gin.Context) {
	h := handlers.NewTaskHandler(r.taskManager, r.executionEngine)
	h.Update(c)
}

func (r *Router) deleteTask(c *gin.Context) {
	h := handlers.NewTaskHandler(r.taskManager, r.executionEngine)
	h.Delete(c)
}

func (r *Router) getNextTask(c *gin.Context) {
	h := handlers.NewTaskHandler(r.taskManager, r.executionEngine)
	h.GetNext(c)
}

func (r *Router) executeTask(c *gin.Context) {
	h := handlers.NewTaskHandler(r.taskManager, r.executionEngine)
	h.Execute(c)
}

func (r *Router) stopTask(c *gin.Context) {
	h := handlers.NewTaskHandler(r.taskManager, r.executionEngine)
	h.Stop(c)
}

func (r *Router) getTaskProgress(c *gin.Context) {
	h := handlers.NewTaskHandler(r.taskManager, r.executionEngine)
	h.GetProgress(c)
}

func (r *Router) getTaskStats(c *gin.Context) {
	h := handlers.NewTaskHandler(r.taskManager, r.executionEngine)
	h.GetStats(c)
}

func (r *Router) listTaskArtifacts(c *gin.Context) {
	h := handlers.NewTaskHandler(r.taskManager, r.executionEngine)
	h.ListArtifacts(c)
}

func (r *Router) getTaskArtifact(c *gin.Context) {
	h := handlers.NewTaskHandler(r.taskManager, r.executionEngine)
	h.GetArtifact(c)
}

// Agent handlers

func (r *Router) listAgents(c *gin.Context) {
	h := handlers.NewAgentHandler(r.agentPool)
	h.List(c)
}

func (r *Router) createAgent(c *gin.Context) {
	h := handlers.NewAgentHandler(r.agentPool)
	h.Create(c)
}

func (r *Router) getAgent(c *gin.Context) {
	h := handlers.NewAgentHandler(r.agentPool)
	h.Get(c)
}

func (r *Router) updateAgent(c *gin.Context) {
	h := handlers.NewAgentHandler(r.agentPool)
	h.Update(c)
}

func (r *Router) deleteAgent(c *gin.Context) {
	h := handlers.NewAgentHandler(r.agentPool)
	h.Delete(c)
}

func (r *Router) listAgentInstances(c *gin.Context) {
	h := handlers.NewAgentHandler(r.agentPool)
	h.ListInstances(c)
}

// Git handlers

func (r *Router) cloneRepo(c *gin.Context) {
	h := handlers.NewGitHandler(r.gitManager)
	h.Clone(c)
}

func (r *Router) listWorktrees(c *gin.Context) {
	h := handlers.NewGitHandler(r.gitManager)
	h.ListWorktrees(c)
}

func (r *Router) createWorktree(c *gin.Context) {
	h := handlers.NewGitHandler(r.gitManager)
	h.CreateWorktree(c)
}

func (r *Router) removeWorktree(c *gin.Context) {
	h := handlers.NewGitHandler(r.gitManager)
	h.RemoveWorktree(c)
}

// MCP handler

func (r *Router) handleMCP(c *gin.Context) {
	r.mcpServer.ServeHTTP(c.Writer, c.Request)
}

// Process handlers

func (r *Router) listProcesses(c *gin.Context) {
	h := handlers.NewProcessHandler(r.processTracker)
	h.ListProcesses(c)
}

func (r *Router) getProcess(c *gin.Context) {
	h := handlers.NewProcessHandler(r.processTracker)
	h.GetProcess(c)
}

func (r *Router) getProcessTree(c *gin.Context) {
	h := handlers.NewProcessHandler(r.processTracker)
	h.GetProcessTree(c)
}

func (r *Router) getProcessGraph(c *gin.Context) {
	h := handlers.NewProcessHandler(r.processTracker)
	h.GetProcessGraph(c)
}

func (r *Router) getActiveProcesses(c *gin.Context) {
	h := handlers.NewProcessHandler(r.processTracker)
	h.GetActiveProcesses(c)
}

func (r *Router) getProcessesByTask(c *gin.Context) {
	h := handlers.NewProcessHandler(r.processTracker)
	h.GetProcessesByTask(c)
}

func (r *Router) getProcessesByInstance(c *gin.Context) {
	h := handlers.NewProcessHandler(r.processTracker)
	h.GetProcessesByInstance(c)
}

func (r *Router) terminateProcess(c *gin.Context) {
	h := handlers.NewProcessHandler(r.processTracker)
	h.TerminateProcess(c)
}

func (r *Router) getProcessStats(c *gin.Context) {
	h := handlers.NewProcessHandler(r.processTracker)
	h.GetProcessStats(c)
}

// WebSocket handler

func (r *Router) handleWebSocket(c *gin.Context) {
	conn, err := r.upgrader.Upgrade(c.Writer, c.Request, nil)
	if err != nil {
		return
	}

	// Register client
	r.wsClientsMu.Lock()
	r.wsClients[conn] = true
	r.wsClientsMu.Unlock()

	defer func() {
		r.wsClientsMu.Lock()
		delete(r.wsClients, conn)
		r.wsClientsMu.Unlock()
		conn.Close()
	}()

	// Send initial process graph
	if r.processTracker != nil {
		graph, err := r.processTracker.GetProcessGraph(nil)
		if err == nil {
			msg := types.WebSocketMessage{
				Type:    "initial_graph",
				Payload: graph,
			}
			data, _ := json.Marshal(msg)
			conn.WriteMessage(websocket.TextMessage, data)
		}
	}

	// Handle incoming messages (e.g., subscribe to specific task)
	for {
		_, message, err := conn.ReadMessage()
		if err != nil {
			break
		}

		// Parse subscription request
		var req struct {
			Action string `json:"action"`
			TaskID string `json:"task_id"`
		}
		if err := json.Unmarshal(message, &req); err != nil {
			continue
		}

		// Handle subscription
		switch req.Action {
		case "subscribe_task":
			// Send current processes for the task
			if r.processTracker != nil {
				processes := r.processTracker.GetProcessesByTask(req.TaskID)
				msg := types.WebSocketMessage{
					Type:    "task_processes",
					Payload: processes,
				}
				data, _ := json.Marshal(msg)
				conn.WriteMessage(websocket.TextMessage, data)
			}
		}
	}
}

// broadcastProcessEvents broadcasts process events to all WebSocket clients.
func (r *Router) broadcastProcessEvents() {
	if r.processTracker == nil {
		return
	}

	// Subscribe to process events
	eventCh := r.processTracker.Subscribe("api_broadcaster")
	defer r.processTracker.Unsubscribe("api_broadcaster")

	for event := range eventCh {
		msg := types.WebSocketMessage{
			Type:    "process_event",
			Payload: event,
		}
		data, err := json.Marshal(msg)
		if err != nil {
			continue
		}

		// Broadcast to all clients
		r.wsClientsMu.RLock()
		for conn := range r.wsClients {
			conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if err := conn.WriteMessage(websocket.TextMessage, data); err != nil {
				// Client will be removed when read fails
				continue
			}
		}
		r.wsClientsMu.RUnlock()
	}
}

// BroadcastMessage sends a message to all WebSocket clients.
func (r *Router) BroadcastMessage(msgType string, payload interface{}) {
	msg := types.WebSocketMessage{
		Type:    msgType,
		Payload: payload,
	}
	data, err := json.Marshal(msg)
	if err != nil {
		return
	}

	r.wsClientsMu.RLock()
	defer r.wsClientsMu.RUnlock()

	for conn := range r.wsClients {
		conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
		conn.WriteMessage(websocket.TextMessage, data)
	}
}
