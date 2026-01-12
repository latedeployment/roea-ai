// Package handlers provides HTTP request handlers.
package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"

	"github.com/roea-ai/roea/internal/core/execution"
	"github.com/roea-ai/roea/internal/core/task"
	"github.com/roea-ai/roea/pkg/types"
)

// TaskHandler handles task-related requests.
type TaskHandler struct {
	taskManager     *task.Manager
	executionEngine *execution.Engine
}

// NewTaskHandler creates a new TaskHandler.
func NewTaskHandler(taskManager *task.Manager, executionEngine *execution.Engine) *TaskHandler {
	return &TaskHandler{
		taskManager:     taskManager,
		executionEngine: executionEngine,
	}
}

// List returns all tasks matching the filter.
func (h *TaskHandler) List(c *gin.Context) {
	filter := &types.TaskFilter{}

	// Parse query params
	if status := c.QueryArray("status"); len(status) > 0 {
		for _, s := range status {
			filter.Status = append(filter.Status, types.TaskStatus(s))
		}
	}
	if agentType := c.Query("agent_type"); agentType != "" {
		filter.AgentType = agentType
	}

	tasks, err := h.taskManager.ListTasks(filter)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, tasks)
}

// Create creates a new task.
func (h *TaskHandler) Create(c *gin.Context) {
	var taskObj types.Task
	if err := c.ShouldBindJSON(&taskObj); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.taskManager.CreateTask(&taskObj); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, taskObj)
}

// Get retrieves a task by ID.
func (h *TaskHandler) Get(c *gin.Context) {
	id := c.Param("id")

	taskObj, err := h.taskManager.GetTask(id)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	if taskObj == nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "task not found"})
		return
	}

	c.JSON(http.StatusOK, taskObj)
}

// Update updates a task.
func (h *TaskHandler) Update(c *gin.Context) {
	id := c.Param("id")

	var update types.TaskUpdate
	if err := c.ShouldBindJSON(&update); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.taskManager.UpdateTask(id, &update); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	// Return updated task
	taskObj, _ := h.taskManager.GetTask(id)
	c.JSON(http.StatusOK, taskObj)
}

// Delete removes a task.
func (h *TaskHandler) Delete(c *gin.Context) {
	id := c.Param("id")

	if err := h.taskManager.DeleteTask(id); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"status": "deleted"})
}

// GetNext returns the next pending task.
func (h *TaskHandler) GetNext(c *gin.Context) {
	agentType := c.Query("agent")

	taskObj, err := h.taskManager.GetNextTask(agentType)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	if taskObj == nil {
		c.JSON(http.StatusOK, nil)
		return
	}

	c.JSON(http.StatusOK, taskObj)
}

// Execute starts execution of a task.
func (h *TaskHandler) Execute(c *gin.Context) {
	id := c.Param("id")

	// Check if async
	async := c.Query("async") == "true"

	if async {
		instanceID, err := h.executionEngine.ExecuteAsync(id)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}
		c.JSON(http.StatusAccepted, gin.H{
			"status":      "started",
			"instance_id": instanceID,
		})
		return
	}

	// Synchronous execution
	result, err := h.executionEngine.Execute(c.Request.Context(), id)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, result)
}

// Stop stops a running task execution.
func (h *TaskHandler) Stop(c *gin.Context) {
	id := c.Param("id")

	if err := h.executionEngine.StopTask(id); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"status": "stopped"})
}

// GetProgress returns the current progress of a task.
func (h *TaskHandler) GetProgress(c *gin.Context) {
	id := c.Param("id")

	progress := h.taskManager.GetProgress(id)
	if progress == nil {
		c.JSON(http.StatusOK, gin.H{
			"task_id":          id,
			"message":          "",
			"percent_complete": 0,
		})
		return
	}

	c.JSON(http.StatusOK, progress)
}

// GetStats returns task statistics.
func (h *TaskHandler) GetStats(c *gin.Context) {
	stats, err := h.taskManager.GetStats()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, stats)
}

// ListArtifacts lists artifacts for a task.
func (h *TaskHandler) ListArtifacts(c *gin.Context) {
	id := c.Param("id")

	artifacts, err := h.taskManager.ListArtifacts(id)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, artifacts)
}

// GetArtifact retrieves a specific artifact.
func (h *TaskHandler) GetArtifact(c *gin.Context) {
	id := c.Param("id")
	name := c.Param("name")

	data, mimetype, err := h.taskManager.GetArtifact(id, name)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	if data == nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "artifact not found"})
		return
	}

	c.Data(http.StatusOK, mimetype, data)
}
