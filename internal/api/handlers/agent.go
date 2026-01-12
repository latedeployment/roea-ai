package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"

	"github.com/roea-ai/roea/internal/core/agent"
	"github.com/roea-ai/roea/pkg/types"
)

// AgentHandler handles agent-related requests.
type AgentHandler struct {
	agentPool *agent.Pool
}

// NewAgentHandler creates a new AgentHandler.
func NewAgentHandler(agentPool *agent.Pool) *AgentHandler {
	return &AgentHandler{
		agentPool: agentPool,
	}
}

// List returns all agent definitions.
func (h *AgentHandler) List(c *gin.Context) {
	agents, err := h.agentPool.ListAgentDefinitions()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, agents)
}

// Create creates a new agent definition.
func (h *AgentHandler) Create(c *gin.Context) {
	var agentDef types.AgentDefinition
	if err := c.ShouldBindJSON(&agentDef); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.agentPool.SaveAgentDefinition(&agentDef); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, agentDef)
}

// Get retrieves an agent definition by ID.
func (h *AgentHandler) Get(c *gin.Context) {
	id := c.Param("id")

	agentDef, err := h.agentPool.GetAgentDefinition(id)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	if agentDef == nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "agent not found"})
		return
	}

	c.JSON(http.StatusOK, agentDef)
}

// Update updates an agent definition.
func (h *AgentHandler) Update(c *gin.Context) {
	id := c.Param("id")

	var agentDef types.AgentDefinition
	if err := c.ShouldBindJSON(&agentDef); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	// Ensure ID matches
	agentDef.ID = id

	if err := h.agentPool.SaveAgentDefinition(&agentDef); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, agentDef)
}

// Delete removes an agent definition.
func (h *AgentHandler) Delete(c *gin.Context) {
	id := c.Param("id")

	if err := h.agentPool.DeleteAgentDefinition(id); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"status": "deleted"})
}

// ListInstances returns all running agent instances.
func (h *AgentHandler) ListInstances(c *gin.Context) {
	instances := h.agentPool.ListInstances()
	c.JSON(http.StatusOK, instances)
}
