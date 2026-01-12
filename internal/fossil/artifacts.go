package fossil

import (
	"database/sql"
	"encoding/base64"
	"fmt"
	"mime"
	"path/filepath"
	"time"

	"github.com/roea-ai/roea/pkg/types"
)

// ArtifactStore handles artifact (attachment) storage.
type ArtifactStore struct {
	store *Store
}

// NewArtifactStore creates a new ArtifactStore.
func NewArtifactStore(store *Store) *ArtifactStore {
	return &ArtifactStore{store: store}
}

// Initialize ensures the artifacts table exists.
func (as *ArtifactStore) Initialize() error {
	as.store.mu.Lock()
	defer as.store.mu.Unlock()

	_, err := as.store.db.Exec(`
		CREATE TABLE IF NOT EXISTS artifact (
			id INTEGER PRIMARY KEY,
			task_id TEXT NOT NULL,
			name TEXT NOT NULL,
			content BLOB,
			size INTEGER,
			mimetype TEXT,
			ctime REAL,
			UNIQUE(task_id, name)
		)
	`)
	if err != nil {
		return fmt.Errorf("failed to create artifact table: %w", err)
	}

	// Create index for task lookups
	_, _ = as.store.db.Exec(`
		CREATE INDEX IF NOT EXISTS idx_artifact_task ON artifact(task_id)
	`)

	return nil
}

// StoreArtifact stores an artifact for a task.
func (as *ArtifactStore) StoreArtifact(taskID string, name string, data []byte) error {
	as.store.mu.Lock()
	defer as.store.mu.Unlock()

	// Determine mimetype from extension
	ext := filepath.Ext(name)
	mimetype := mime.TypeByExtension(ext)
	if mimetype == "" {
		mimetype = "application/octet-stream"
	}

	now := float64(time.Now().Unix())

	_, err := as.store.db.Exec(`
		INSERT INTO artifact (task_id, name, content, size, mimetype, ctime)
		VALUES (?, ?, ?, ?, ?, ?)
		ON CONFLICT(task_id, name) DO UPDATE SET
			content = excluded.content,
			size = excluded.size,
			mimetype = excluded.mimetype
	`, taskID, name, data, len(data), mimetype, now)

	if err != nil {
		return fmt.Errorf("failed to store artifact: %w", err)
	}

	return nil
}

// StoreArtifactBase64 stores base64-encoded artifact data.
func (as *ArtifactStore) StoreArtifactBase64(taskID string, name string, b64Data string) error {
	data, err := base64.StdEncoding.DecodeString(b64Data)
	if err != nil {
		return fmt.Errorf("failed to decode base64: %w", err)
	}
	return as.StoreArtifact(taskID, name, data)
}

// GetArtifact retrieves an artifact by task ID and name.
func (as *ArtifactStore) GetArtifact(taskID string, name string) ([]byte, string, error) {
	as.store.mu.RLock()
	defer as.store.mu.RUnlock()

	var content []byte
	var mimetype string
	err := as.store.db.QueryRow(
		"SELECT content, mimetype FROM artifact WHERE task_id = ? AND name = ?",
		taskID, name,
	).Scan(&content, &mimetype)

	if err == sql.ErrNoRows {
		return nil, "", nil
	}
	if err != nil {
		return nil, "", fmt.Errorf("failed to get artifact: %w", err)
	}

	return content, mimetype, nil
}

// ListArtifacts returns all artifacts for a task.
func (as *ArtifactStore) ListArtifacts(taskID string) ([]*types.TaskArtifact, error) {
	as.store.mu.RLock()
	defer as.store.mu.RUnlock()

	rows, err := as.store.db.Query(
		"SELECT name, size, mimetype, ctime FROM artifact WHERE task_id = ?",
		taskID,
	)
	if err != nil {
		return nil, fmt.Errorf("failed to list artifacts: %w", err)
	}
	defer rows.Close()

	var artifacts []*types.TaskArtifact
	for rows.Next() {
		var artifact types.TaskArtifact
		var ctimeUnix float64
		var mimetype sql.NullString

		if err := rows.Scan(&artifact.Name, &artifact.Size, &mimetype, &ctimeUnix); err != nil {
			continue
		}

		artifact.TaskID = taskID
		artifact.CreatedAt = time.Unix(int64(ctimeUnix), 0)
		if mimetype.Valid {
			artifact.MimeType = mimetype.String
		}

		artifacts = append(artifacts, &artifact)
	}

	return artifacts, nil
}

// DeleteArtifact removes an artifact.
func (as *ArtifactStore) DeleteArtifact(taskID string, name string) error {
	as.store.mu.Lock()
	defer as.store.mu.Unlock()

	result, err := as.store.db.Exec(
		"DELETE FROM artifact WHERE task_id = ? AND name = ?",
		taskID, name,
	)
	if err != nil {
		return fmt.Errorf("failed to delete artifact: %w", err)
	}

	rows, _ := result.RowsAffected()
	if rows == 0 {
		return fmt.Errorf("artifact not found")
	}

	return nil
}

// DeleteTaskArtifacts removes all artifacts for a task.
func (as *ArtifactStore) DeleteTaskArtifacts(taskID string) error {
	as.store.mu.Lock()
	defer as.store.mu.Unlock()

	_, err := as.store.db.Exec("DELETE FROM artifact WHERE task_id = ?", taskID)
	if err != nil {
		return fmt.Errorf("failed to delete task artifacts: %w", err)
	}

	return nil
}

// GetArtifactBase64 retrieves artifact as base64.
func (as *ArtifactStore) GetArtifactBase64(taskID string, name string) (string, string, error) {
	data, mimetype, err := as.GetArtifact(taskID, name)
	if err != nil {
		return "", "", err
	}
	if data == nil {
		return "", "", nil
	}

	return base64.StdEncoding.EncodeToString(data), mimetype, nil
}

// GetTotalArtifactSize returns total artifact size for a task.
func (as *ArtifactStore) GetTotalArtifactSize(taskID string) (int64, error) {
	as.store.mu.RLock()
	defer as.store.mu.RUnlock()

	var total sql.NullInt64
	err := as.store.db.QueryRow(
		"SELECT SUM(size) FROM artifact WHERE task_id = ?",
		taskID,
	).Scan(&total)

	if err != nil {
		return 0, fmt.Errorf("failed to get artifact size: %w", err)
	}

	if total.Valid {
		return total.Int64, nil
	}
	return 0, nil
}
