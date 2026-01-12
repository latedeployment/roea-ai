package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"

	"github.com/roea-ai/roea/internal/core/git"
)

// GitHandler handles Git-related requests.
type GitHandler struct {
	gitManager *git.Manager
}

// NewGitHandler creates a new GitHandler.
func NewGitHandler(gitManager *git.Manager) *GitHandler {
	return &GitHandler{
		gitManager: gitManager,
	}
}

// CloneRequest represents a clone request.
type CloneRequest struct {
	URL      string `json:"url" binding:"required"`
	DestPath string `json:"dest_path" binding:"required"`
}

// Clone clones a repository.
func (h *GitHandler) Clone(c *gin.Context) {
	var req CloneRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.gitManager.CloneRepository(req.URL, req.DestPath); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"status": "cloned",
		"path":   req.DestPath,
	})
}

// ListWorktrees returns all active worktrees.
func (h *GitHandler) ListWorktrees(c *gin.Context) {
	worktrees := h.gitManager.ListWorktrees()
	c.JSON(http.StatusOK, worktrees)
}

// WorktreeRequest represents a worktree creation request.
type WorktreeRequest struct {
	TaskID   string `json:"task_id" binding:"required"`
	RepoPath string `json:"repo_path" binding:"required"`
	Branch   string `json:"branch"`
}

// CreateWorktree creates a new worktree.
func (h *GitHandler) CreateWorktree(c *gin.Context) {
	var req WorktreeRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	wt, err := h.gitManager.CreateWorktree(req.TaskID, req.RepoPath, req.Branch)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, wt)
}

// RemoveWorktree removes a worktree.
func (h *GitHandler) RemoveWorktree(c *gin.Context) {
	taskID := c.Param("taskId")

	if err := h.gitManager.RemoveWorktree(taskID); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"status": "removed"})
}
