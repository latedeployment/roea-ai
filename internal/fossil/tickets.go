package fossil

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"github.com/roea-ai/roea/pkg/types"
)

// TicketStore handles ticket (task) operations.
type TicketStore struct {
	store *Store
}

// NewTicketStore creates a new TicketStore.
func NewTicketStore(store *Store) *TicketStore {
	return &TicketStore{store: store}
}

// CreateTask creates a new task (ticket).
func (ts *TicketStore) CreateTask(task *types.Task) error {
	ts.store.mu.Lock()
	defer ts.store.mu.Unlock()

	now := time.Now()
	if task.CreatedAt.IsZero() {
		task.CreatedAt = now
	}

	// Generate UUID if not provided
	if task.ID == "" {
		task.ID = generateUUID()
	}

	// Serialize labels
	labelsJSON := ""
	if len(task.Labels) > 0 {
		data, _ := json.Marshal(task.Labels)
		labelsJSON = string(data)
	}

	// Serialize secrets if present
	secretsJSON := ""
	if task.Secrets != nil {
		data, _ := json.Marshal(task.Secrets)
		secretsJSON = string(data)
	}

	_, err := ts.store.db.Exec(`
		INSERT INTO ticket (
			tkt_uuid, title, comment, status, type, priority,
			agent_type, execution_mode, model, repo_url, branch, worktree,
			secrets_encrypted, parent_id, labels,
			tkt_ctime, tkt_mtime
		) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
	`,
		task.ID,
		task.Title,
		task.Description,
		string(task.Status),
		task.AgentType,
		task.Priority,
		task.AgentType,
		string(task.ExecutionMode),
		task.Model,
		task.RepoURL,
		task.Branch,
		task.Worktree,
		secretsJSON,
		task.ParentID,
		labelsJSON,
		task.CreatedAt.Format(time.RFC3339),
		now.Format(time.RFC3339),
	)

	if err != nil {
		return fmt.Errorf("failed to create task: %w", err)
	}

	return nil
}

// GetTask retrieves a task by ID.
func (ts *TicketStore) GetTask(id string) (*types.Task, error) {
	ts.store.mu.RLock()
	defer ts.store.mu.RUnlock()

	row := ts.store.db.QueryRow(`
		SELECT
			tkt_uuid, title, comment, status, priority,
			agent_type, execution_mode, model, repo_url, branch, worktree,
			secrets_encrypted, parent_id, labels, result, error_message,
			tkt_ctime, tkt_mtime, started_at, completed_at
		FROM ticket
		WHERE tkt_uuid = ?
	`, id)

	return ts.scanTask(row)
}

// UpdateTask updates an existing task.
func (ts *TicketStore) UpdateTask(id string, update *types.TaskUpdate) error {
	ts.store.mu.Lock()
	defer ts.store.mu.Unlock()

	// Build dynamic update query
	var setClauses []string
	var args []interface{}

	if update.Title != nil {
		setClauses = append(setClauses, "title = ?")
		args = append(args, *update.Title)
	}
	if update.Description != nil {
		setClauses = append(setClauses, "comment = ?")
		args = append(args, *update.Description)
	}
	if update.Status != nil {
		setClauses = append(setClauses, "status = ?")
		args = append(args, string(*update.Status))
	}
	if update.AgentType != nil {
		setClauses = append(setClauses, "agent_type = ?")
		args = append(args, *update.AgentType)
	}
	if update.ExecutionMode != nil {
		setClauses = append(setClauses, "execution_mode = ?")
		args = append(args, string(*update.ExecutionMode))
	}
	if update.Model != nil {
		setClauses = append(setClauses, "model = ?")
		args = append(args, *update.Model)
	}
	if update.Priority != nil {
		setClauses = append(setClauses, "priority = ?")
		args = append(args, *update.Priority)
	}
	if update.Labels != nil {
		data, _ := json.Marshal(update.Labels)
		setClauses = append(setClauses, "labels = ?")
		args = append(args, string(data))
	}
	if update.Secrets != nil {
		data, _ := json.Marshal(update.Secrets)
		setClauses = append(setClauses, "secrets_encrypted = ?")
		args = append(args, string(data))
	}
	if update.StartedAt != nil {
		setClauses = append(setClauses, "started_at = ?")
		args = append(args, update.StartedAt.Format(time.RFC3339))
	}
	if update.CompletedAt != nil {
		setClauses = append(setClauses, "completed_at = ?")
		args = append(args, update.CompletedAt.Format(time.RFC3339))
	}
	if update.Result != nil {
		setClauses = append(setClauses, "result = ?")
		args = append(args, *update.Result)
	}
	if update.ErrorMessage != nil {
		setClauses = append(setClauses, "error_message = ?")
		args = append(args, *update.ErrorMessage)
	}

	if len(setClauses) == 0 {
		return nil // Nothing to update
	}

	// Always update mtime
	setClauses = append(setClauses, "tkt_mtime = ?")
	args = append(args, time.Now().Format(time.RFC3339))

	// Add ID to args
	args = append(args, id)

	query := fmt.Sprintf("UPDATE ticket SET %s WHERE tkt_uuid = ?",
		strings.Join(setClauses, ", "))

	result, err := ts.store.db.Exec(query, args...)
	if err != nil {
		return fmt.Errorf("failed to update task: %w", err)
	}

	rows, _ := result.RowsAffected()
	if rows == 0 {
		return fmt.Errorf("task not found: %s", id)
	}

	return nil
}

// DeleteTask deletes a task.
func (ts *TicketStore) DeleteTask(id string) error {
	ts.store.mu.Lock()
	defer ts.store.mu.Unlock()

	result, err := ts.store.db.Exec("DELETE FROM ticket WHERE tkt_uuid = ?", id)
	if err != nil {
		return fmt.Errorf("failed to delete task: %w", err)
	}

	rows, _ := result.RowsAffected()
	if rows == 0 {
		return fmt.Errorf("task not found: %s", id)
	}

	return nil
}

// ListTasks retrieves tasks matching the filter.
func (ts *TicketStore) ListTasks(filter *types.TaskFilter) ([]*types.Task, error) {
	ts.store.mu.RLock()
	defer ts.store.mu.RUnlock()

	var whereClauses []string
	var args []interface{}

	if filter != nil {
		if len(filter.Status) > 0 {
			placeholders := make([]string, len(filter.Status))
			for i, s := range filter.Status {
				placeholders[i] = "?"
				args = append(args, string(s))
			}
			whereClauses = append(whereClauses, fmt.Sprintf("status IN (%s)", strings.Join(placeholders, ",")))
		}

		if filter.AgentType != "" {
			whereClauses = append(whereClauses, "agent_type = ?")
			args = append(args, filter.AgentType)
		}

		if filter.ParentID != nil {
			whereClauses = append(whereClauses, "parent_id = ?")
			args = append(args, *filter.ParentID)
		}
	}

	query := `
		SELECT
			tkt_uuid, title, comment, status, priority,
			agent_type, execution_mode, model, repo_url, branch, worktree,
			secrets_encrypted, parent_id, labels, result, error_message,
			tkt_ctime, tkt_mtime, started_at, completed_at
		FROM ticket
	`

	if len(whereClauses) > 0 {
		query += " WHERE " + strings.Join(whereClauses, " AND ")
	}

	query += " ORDER BY priority ASC, tkt_ctime DESC"

	if filter != nil && filter.Limit > 0 {
		query += fmt.Sprintf(" LIMIT %d", filter.Limit)
		if filter.Offset > 0 {
			query += fmt.Sprintf(" OFFSET %d", filter.Offset)
		}
	}

	rows, err := ts.store.db.Query(query, args...)
	if err != nil {
		return nil, fmt.Errorf("failed to query tasks: %w", err)
	}
	defer rows.Close()

	var tasks []*types.Task
	for rows.Next() {
		task, err := ts.scanTaskRows(rows)
		if err != nil {
			return nil, err
		}
		tasks = append(tasks, task)
	}

	return tasks, nil
}

// GetNextTask returns the next pending task for an agent type.
func (ts *TicketStore) GetNextTask(agentType string) (*types.Task, error) {
	ts.store.mu.RLock()
	defer ts.store.mu.RUnlock()

	query := `
		SELECT
			tkt_uuid, title, comment, status, priority,
			agent_type, execution_mode, model, repo_url, branch, worktree,
			secrets_encrypted, parent_id, labels, result, error_message,
			tkt_ctime, tkt_mtime, started_at, completed_at
		FROM ticket
		WHERE status = 'pending'
	`

	var args []interface{}
	if agentType != "" {
		query += " AND agent_type = ?"
		args = append(args, agentType)
	}

	query += " ORDER BY priority ASC, tkt_ctime ASC LIMIT 1"

	row := ts.store.db.QueryRow(query, args...)
	return ts.scanTask(row)
}

// CountTasks counts tasks matching the filter.
func (ts *TicketStore) CountTasks(filter *types.TaskFilter) (int, error) {
	ts.store.mu.RLock()
	defer ts.store.mu.RUnlock()

	var whereClauses []string
	var args []interface{}

	if filter != nil {
		if len(filter.Status) > 0 {
			placeholders := make([]string, len(filter.Status))
			for i, s := range filter.Status {
				placeholders[i] = "?"
				args = append(args, string(s))
			}
			whereClauses = append(whereClauses, fmt.Sprintf("status IN (%s)", strings.Join(placeholders, ",")))
		}

		if filter.AgentType != "" {
			whereClauses = append(whereClauses, "agent_type = ?")
			args = append(args, filter.AgentType)
		}
	}

	query := "SELECT COUNT(*) FROM ticket"
	if len(whereClauses) > 0 {
		query += " WHERE " + strings.Join(whereClauses, " AND ")
	}

	var count int
	err := ts.store.db.QueryRow(query, args...).Scan(&count)
	if err != nil {
		return 0, fmt.Errorf("failed to count tasks: %w", err)
	}

	return count, nil
}

// scanTask scans a single task from a row.
func (ts *TicketStore) scanTask(row *sql.Row) (*types.Task, error) {
	var task types.Task
	var status, execMode, labelsJSON, secretsJSON sql.NullString
	var parentID, result, errorMsg sql.NullString
	var startedAt, completedAt sql.NullString
	var ctimeRaw, mtimeRaw interface{}

	err := row.Scan(
		&task.ID,
		&task.Title,
		&task.Description,
		&status,
		&task.Priority,
		&task.AgentType,
		&execMode,
		&task.Model,
		&task.RepoURL,
		&task.Branch,
		&task.Worktree,
		&secretsJSON,
		&parentID,
		&labelsJSON,
		&result,
		&errorMsg,
		&ctimeRaw,
		&mtimeRaw,
		&startedAt,
		&completedAt,
	)

	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("failed to scan task: %w", err)
	}

	// Parse nullable fields
	if status.Valid {
		task.Status = types.TaskStatus(status.String)
	}
	if execMode.Valid {
		task.ExecutionMode = types.ExecutionMode(execMode.String)
	}
	if parentID.Valid {
		task.ParentID = &parentID.String
	}
	if result.Valid {
		task.Result = result.String
	}
	if errorMsg.Valid {
		task.ErrorMessage = errorMsg.String
	}

	// Parse labels
	if labelsJSON.Valid && labelsJSON.String != "" {
		json.Unmarshal([]byte(labelsJSON.String), &task.Labels)
	}

	// Parse secrets
	if secretsJSON.Valid && secretsJSON.String != "" {
		var secrets types.EncryptedPayload
		if err := json.Unmarshal([]byte(secretsJSON.String), &secrets); err == nil {
			task.Secrets = &secrets
		}
	}

	// Parse timestamps (can be time.Time or string depending on SQLite driver)
	task.CreatedAt = parseTimestamp(ctimeRaw)
	_ = mtimeRaw // unused but scanned
	if startedAt.Valid && startedAt.String != "" {
		t, _ := time.Parse(time.RFC3339, startedAt.String)
		task.StartedAt = &t
	}
	if completedAt.Valid && completedAt.String != "" {
		t, _ := time.Parse(time.RFC3339, completedAt.String)
		task.CompletedAt = &t
	}

	return &task, nil
}

// scanTaskRows scans a task from a rows result.
func (ts *TicketStore) scanTaskRows(rows *sql.Rows) (*types.Task, error) {
	var task types.Task
	var status, execMode, labelsJSON, secretsJSON sql.NullString
	var parentID, result, errorMsg sql.NullString
	var startedAt, completedAt sql.NullString
	var ctimeRaw, mtimeRaw interface{}

	err := rows.Scan(
		&task.ID,
		&task.Title,
		&task.Description,
		&status,
		&task.Priority,
		&task.AgentType,
		&execMode,
		&task.Model,
		&task.RepoURL,
		&task.Branch,
		&task.Worktree,
		&secretsJSON,
		&parentID,
		&labelsJSON,
		&result,
		&errorMsg,
		&ctimeRaw,
		&mtimeRaw,
		&startedAt,
		&completedAt,
	)

	if err != nil {
		return nil, fmt.Errorf("failed to scan task: %w", err)
	}

	// Parse nullable fields (same as scanTask)
	if status.Valid {
		task.Status = types.TaskStatus(status.String)
	}
	if execMode.Valid {
		task.ExecutionMode = types.ExecutionMode(execMode.String)
	}
	if parentID.Valid {
		task.ParentID = &parentID.String
	}
	if result.Valid {
		task.Result = result.String
	}
	if errorMsg.Valid {
		task.ErrorMessage = errorMsg.String
	}

	if labelsJSON.Valid && labelsJSON.String != "" {
		json.Unmarshal([]byte(labelsJSON.String), &task.Labels)
	}

	if secretsJSON.Valid && secretsJSON.String != "" {
		var secrets types.EncryptedPayload
		if err := json.Unmarshal([]byte(secretsJSON.String), &secrets); err == nil {
			task.Secrets = &secrets
		}
	}

	// Parse timestamps (can be time.Time or string depending on SQLite driver)
	task.CreatedAt = parseTimestamp(ctimeRaw)
	_ = mtimeRaw // unused but scanned
	if startedAt.Valid && startedAt.String != "" {
		t, _ := time.Parse(time.RFC3339, startedAt.String)
		task.StartedAt = &t
	}
	if completedAt.Valid && completedAt.String != "" {
		t, _ := time.Parse(time.RFC3339, completedAt.String)
		task.CompletedAt = &t
	}

	return &task, nil
}

// parseTimestamp handles both time.Time and string timestamp values from SQLite.
func parseTimestamp(v interface{}) time.Time {
	if v == nil {
		return time.Time{}
	}
	switch t := v.(type) {
	case time.Time:
		return t
	case string:
		if parsed, err := time.Parse(time.RFC3339, t); err == nil {
			return parsed
		}
	}
	return time.Time{}
}

// generateUUID generates a simple UUID for tickets.
func generateUUID() string {
	return fmt.Sprintf("%x", time.Now().UnixNano())
}
