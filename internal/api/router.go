// Package api provides the REST API for Roea.
package api

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/gorilla/websocket"

	"github.com/roea-ai/roea/internal/api/handlers"
	"github.com/roea-ai/roea/internal/core/agent"
	"github.com/roea-ai/roea/internal/core/execution"
	"github.com/roea-ai/roea/internal/core/git"
	"github.com/roea-ai/roea/internal/core/task"
	"github.com/roea-ai/roea/internal/mcp"
)

// Router holds all API dependencies and routes.
type Router struct {
	engine          *gin.Engine
	taskManager     *task.Manager
	agentPool       *agent.Pool
	executionEngine *execution.Engine
	gitManager      *git.Manager
	mcpServer       *mcp.Server

	// WebSocket upgrader
	upgrader websocket.Upgrader
}

// NewRouter creates a new API router.
func NewRouter(
	taskManager *task.Manager,
	agentPool *agent.Pool,
	executionEngine *execution.Engine,
	gitManager *git.Manager,
	mcpServer *mcp.Server,
) *Router {
	r := &Router{
		engine:          gin.Default(),
		taskManager:     taskManager,
		agentPool:       agentPool,
		executionEngine: executionEngine,
		gitManager:      gitManager,
		mcpServer:       mcpServer,
		upgrader: websocket.Upgrader{
			CheckOrigin: func(r *http.Request) bool {
				return true // Allow all origins for development
			},
		},
	}

	r.setupRoutes()
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

// WebSocket handler

func (r *Router) handleWebSocket(c *gin.Context) {
	conn, err := r.upgrader.Upgrade(c.Writer, c.Request, nil)
	if err != nil {
		return
	}
	defer conn.Close()

	// Handle WebSocket connection for real-time updates
	// This is a placeholder for real-time task progress streaming
	for {
		_, _, err := conn.ReadMessage()
		if err != nil {
			break
		}
	}
}
