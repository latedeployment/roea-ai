package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"

	"github.com/roea-ai/roea/internal/core/process"
	"github.com/roea-ai/roea/pkg/types"
)

// ProcessHandler handles process-related API requests.
type ProcessHandler struct {
	tracker *process.Tracker
}

// NewProcessHandler creates a new ProcessHandler.
func NewProcessHandler(tracker *process.Tracker) *ProcessHandler {
	return &ProcessHandler{tracker: tracker}
}

// ListProcesses returns all processes matching the filter.
func (h *ProcessHandler) ListProcesses(c *gin.Context) {
	filter := &types.ProcessFilter{}

	// Parse query parameters
	if taskID := c.Query("task_id"); taskID != "" {
		filter.TaskID = taskID
	}

	if instanceID := c.Query("instance_id"); instanceID != "" {
		filter.InstanceID = instanceID
	}

	if parentID := c.Query("parent_id"); parentID != "" {
		filter.ParentID = parentID
	}

	if status := c.QueryArray("status"); len(status) > 0 {
		for _, s := range status {
			filter.Status = append(filter.Status, types.ProcessStatus(s))
		}
	}

	processes, err := h.tracker.ListProcesses(filter)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, processes)
}

// GetProcess returns a specific process by ID.
func (h *ProcessHandler) GetProcess(c *gin.Context) {
	id := c.Param("id")

	process, err := h.tracker.GetProcess(id)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, process)
}

// GetProcessTree returns the process tree for a root process.
func (h *ProcessHandler) GetProcessTree(c *gin.Context) {
	id := c.Param("id")

	tree, err := h.tracker.GetProcessTree(id)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, tree)
}

// GetProcessGraph returns the process graph data for visualization.
func (h *ProcessHandler) GetProcessGraph(c *gin.Context) {
	filter := &types.ProcessFilter{}

	// Parse query parameters
	if taskID := c.Query("task_id"); taskID != "" {
		filter.TaskID = taskID
	}

	if instanceID := c.Query("instance_id"); instanceID != "" {
		filter.InstanceID = instanceID
	}

	// For graph visualization, we might want to limit results
	if c.Query("active_only") == "true" {
		filter.Status = []types.ProcessStatus{
			types.ProcessStarting,
			types.ProcessRunning,
		}
	}

	graph, err := h.tracker.GetProcessGraph(filter)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, graph)
}

// GetActiveProcesses returns all currently running processes.
func (h *ProcessHandler) GetActiveProcesses(c *gin.Context) {
	processes := h.tracker.GetActiveProcesses()
	c.JSON(http.StatusOK, processes)
}

// GetProcessesByTask returns all processes for a specific task.
func (h *ProcessHandler) GetProcessesByTask(c *gin.Context) {
	taskID := c.Param("taskId")

	processes := h.tracker.GetProcessesByTask(taskID)
	c.JSON(http.StatusOK, processes)
}

// GetProcessesByInstance returns all processes for a specific agent instance.
func (h *ProcessHandler) GetProcessesByInstance(c *gin.Context) {
	instanceID := c.Param("instanceId")

	processes := h.tracker.GetProcessesByInstance(instanceID)
	c.JSON(http.StatusOK, processes)
}

// TerminateProcess terminates a process and all its children.
func (h *ProcessHandler) TerminateProcess(c *gin.Context) {
	id := c.Param("id")

	if err := h.tracker.TerminateProcess(id); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "Process terminated"})
}

// GetProcessStats returns aggregated process statistics.
func (h *ProcessHandler) GetProcessStats(c *gin.Context) {
	filter := &types.ProcessFilter{}

	if taskID := c.Query("task_id"); taskID != "" {
		filter.TaskID = taskID
	}

	graph, err := h.tracker.GetProcessGraph(filter)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, graph.Stats)
}
