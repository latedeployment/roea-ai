// Package git provides Git repository management.
package git

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"

	"github.com/roea-ai/roea/pkg/types"
)

// Manager handles Git operations for task execution.
type Manager struct {
	config       *types.GitConfig
	worktreeBase string

	// Track active worktrees
	worktreesMu sync.RWMutex
	worktrees   map[string]*Worktree
}

// Worktree represents a Git worktree for a task.
type Worktree struct {
	TaskID     string
	Path       string
	Branch     string
	RepoPath   string
	CreatedAt  string
}

// NewManager creates a new Git Manager.
func NewManager(config *types.GitConfig, worktreeBase string) *Manager {
	return &Manager{
		config:       config,
		worktreeBase: worktreeBase,
		worktrees:    make(map[string]*Worktree),
	}
}

// CloneRepository clones a repository to a local path.
func (m *Manager) CloneRepository(url string, destPath string) error {
	// Ensure parent directory exists
	if err := os.MkdirAll(filepath.Dir(destPath), 0755); err != nil {
		return fmt.Errorf("failed to create directory: %w", err)
	}

	cmd := exec.Command("git", "clone", url, destPath)
	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("git clone failed: %s: %w", string(output), err)
	}

	return nil
}

// PullRepository pulls latest changes from remote.
func (m *Manager) PullRepository(repoPath string, remote string, branch string) error {
	if remote == "" {
		remote = m.config.DefaultRemote
	}
	if branch == "" {
		branch = "main"
	}

	cmd := exec.Command("git", "pull", remote, branch)
	cmd.Dir = repoPath
	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("git pull failed: %s: %w", string(output), err)
	}

	return nil
}

// CreateWorktree creates a Git worktree for a task.
func (m *Manager) CreateWorktree(taskID string, repoPath string, branch string) (*Worktree, error) {
	m.worktreesMu.Lock()
	defer m.worktreesMu.Unlock()

	// Check if worktree already exists for this task
	if wt, ok := m.worktrees[taskID]; ok {
		return wt, nil
	}

	// Generate worktree path
	wtPath := filepath.Join(m.worktreeBase, taskID)

	// Create branch name if not specified
	if branch == "" {
		branch = m.config.BranchPrefix + taskID
	}

	// Ensure worktree base exists
	if err := os.MkdirAll(m.worktreeBase, 0755); err != nil {
		return nil, fmt.Errorf("failed to create worktree base: %w", err)
	}

	// Check if branch exists
	branchExists := m.branchExists(repoPath, branch)

	var cmd *exec.Cmd
	if branchExists {
		// Checkout existing branch
		cmd = exec.Command("git", "worktree", "add", wtPath, branch)
	} else {
		// Create new branch
		cmd = exec.Command("git", "worktree", "add", "-b", branch, wtPath)
	}
	cmd.Dir = repoPath

	output, err := cmd.CombinedOutput()
	if err != nil {
		// Check if worktree already exists
		if strings.Contains(string(output), "already exists") {
			// Use existing worktree
		} else {
			return nil, fmt.Errorf("git worktree add failed: %s: %w", string(output), err)
		}
	}

	wt := &Worktree{
		TaskID:   taskID,
		Path:     wtPath,
		Branch:   branch,
		RepoPath: repoPath,
	}

	m.worktrees[taskID] = wt
	return wt, nil
}

// RemoveWorktree removes a Git worktree.
func (m *Manager) RemoveWorktree(taskID string) error {
	m.worktreesMu.Lock()
	defer m.worktreesMu.Unlock()

	wt, ok := m.worktrees[taskID]
	if !ok {
		return nil // Already removed
	}

	// Remove worktree
	cmd := exec.Command("git", "worktree", "remove", wt.Path, "--force")
	cmd.Dir = wt.RepoPath
	if output, err := cmd.CombinedOutput(); err != nil {
		// Try to remove directory directly if git worktree fails
		if err := os.RemoveAll(wt.Path); err != nil {
			return fmt.Errorf("failed to remove worktree: %s: %w", string(output), err)
		}
	}

	delete(m.worktrees, taskID)
	return nil
}

// GetWorktree retrieves a worktree by task ID.
func (m *Manager) GetWorktree(taskID string) *Worktree {
	m.worktreesMu.RLock()
	defer m.worktreesMu.RUnlock()

	return m.worktrees[taskID]
}

// ListWorktrees returns all active worktrees.
func (m *Manager) ListWorktrees() []*Worktree {
	m.worktreesMu.RLock()
	defer m.worktreesMu.RUnlock()

	wts := make([]*Worktree, 0, len(m.worktrees))
	for _, wt := range m.worktrees {
		wts = append(wts, wt)
	}
	return wts
}

// CreateBranch creates a new branch.
func (m *Manager) CreateBranch(repoPath string, branchName string, baseBranch string) error {
	if baseBranch == "" {
		baseBranch = "main"
	}

	cmd := exec.Command("git", "checkout", "-b", branchName, baseBranch)
	cmd.Dir = repoPath
	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("git checkout -b failed: %s: %w", string(output), err)
	}

	return nil
}

// CheckoutBranch switches to an existing branch.
func (m *Manager) CheckoutBranch(repoPath string, branchName string) error {
	cmd := exec.Command("git", "checkout", branchName)
	cmd.Dir = repoPath
	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("git checkout failed: %s: %w", string(output), err)
	}

	return nil
}

// CommitChanges commits all changes with a message.
func (m *Manager) CommitChanges(repoPath string, message string) error {
	// Stage all changes
	addCmd := exec.Command("git", "add", "-A")
	addCmd.Dir = repoPath
	if output, err := addCmd.CombinedOutput(); err != nil {
		return fmt.Errorf("git add failed: %s: %w", string(output), err)
	}

	// Check if there are changes to commit
	statusCmd := exec.Command("git", "status", "--porcelain")
	statusCmd.Dir = repoPath
	statusOutput, _ := statusCmd.Output()
	if len(statusOutput) == 0 {
		return nil // Nothing to commit
	}

	// Commit
	commitCmd := exec.Command("git", "commit", "-m", message)
	commitCmd.Dir = repoPath
	if output, err := commitCmd.CombinedOutput(); err != nil {
		return fmt.Errorf("git commit failed: %s: %w", string(output), err)
	}

	return nil
}

// PushBranch pushes a branch to remote.
func (m *Manager) PushBranch(repoPath string, branch string, remote string) error {
	if remote == "" {
		remote = m.config.DefaultRemote
	}

	cmd := exec.Command("git", "push", "-u", remote, branch)
	cmd.Dir = repoPath
	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("git push failed: %s: %w", string(output), err)
	}

	return nil
}

// GetCurrentBranch returns the current branch name.
func (m *Manager) GetCurrentBranch(repoPath string) (string, error) {
	cmd := exec.Command("git", "rev-parse", "--abbrev-ref", "HEAD")
	cmd.Dir = repoPath
	output, err := cmd.Output()
	if err != nil {
		return "", fmt.Errorf("failed to get current branch: %w", err)
	}

	return strings.TrimSpace(string(output)), nil
}

// GetRepoStatus returns the repository status.
func (m *Manager) GetRepoStatus(repoPath string) (string, error) {
	cmd := exec.Command("git", "status", "--short")
	cmd.Dir = repoPath
	output, err := cmd.Output()
	if err != nil {
		return "", fmt.Errorf("failed to get status: %w", err)
	}

	return string(output), nil
}

// branchExists checks if a branch exists.
func (m *Manager) branchExists(repoPath string, branch string) bool {
	cmd := exec.Command("git", "rev-parse", "--verify", branch)
	cmd.Dir = repoPath
	return cmd.Run() == nil
}

// GetDiff returns the diff of uncommitted changes.
func (m *Manager) GetDiff(repoPath string) (string, error) {
	cmd := exec.Command("git", "diff")
	cmd.Dir = repoPath
	output, err := cmd.Output()
	if err != nil {
		return "", fmt.Errorf("failed to get diff: %w", err)
	}

	return string(output), nil
}

// FetchRemote fetches from remote.
func (m *Manager) FetchRemote(repoPath string, remote string) error {
	if remote == "" {
		remote = m.config.DefaultRemote
	}

	cmd := exec.Command("git", "fetch", remote)
	cmd.Dir = repoPath
	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("git fetch failed: %s: %w", string(output), err)
	}

	return nil
}
