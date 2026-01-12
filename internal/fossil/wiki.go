package fossil

import (
	"database/sql"
	"fmt"
	"strings"
	"time"

	"gopkg.in/yaml.v3"

	"github.com/roea-ai/roea/pkg/types"
)

// WikiStore handles wiki page operations for agent definitions.
type WikiStore struct {
	store *Store
}

// NewWikiStore creates a new WikiStore.
func NewWikiStore(store *Store) *WikiStore {
	return &WikiStore{store: store}
}

// Initialize ensures the wiki table exists.
func (ws *WikiStore) Initialize() error {
	ws.store.mu.Lock()
	defer ws.store.mu.Unlock()

	// Check if wiki table exists, if not create it
	_, err := ws.store.db.Exec(`
		CREATE TABLE IF NOT EXISTS wiki (
			id INTEGER PRIMARY KEY,
			name TEXT UNIQUE NOT NULL,
			content TEXT,
			mimetype TEXT DEFAULT 'text/x-markdown',
			mtime REAL,
			ctime REAL
		)
	`)
	if err != nil {
		return fmt.Errorf("failed to create wiki table: %w", err)
	}

	return nil
}

// agentDefPrefix is the prefix for agent definition wiki pages.
const agentDefPrefix = "AgentDef_"

// SaveAgentDefinition saves an agent definition as a wiki page.
func (ws *WikiStore) SaveAgentDefinition(agent *types.AgentDefinition) error {
	ws.store.mu.Lock()
	defer ws.store.mu.Unlock()

	// Serialize to YAML
	content, err := yaml.Marshal(agent)
	if err != nil {
		return fmt.Errorf("failed to marshal agent definition: %w", err)
	}

	pageName := agentDefPrefix + agent.ID
	now := float64(time.Now().Unix())

	// Upsert the wiki page
	_, err = ws.store.db.Exec(`
		INSERT INTO wiki (name, content, mimetype, mtime, ctime)
		VALUES (?, ?, 'text/x-yaml', ?, ?)
		ON CONFLICT(name) DO UPDATE SET
			content = excluded.content,
			mtime = excluded.mtime
	`, pageName, string(content), now, now)

	if err != nil {
		return fmt.Errorf("failed to save agent definition: %w", err)
	}

	return nil
}

// GetAgentDefinition retrieves an agent definition by ID.
func (ws *WikiStore) GetAgentDefinition(id string) (*types.AgentDefinition, error) {
	ws.store.mu.RLock()
	defer ws.store.mu.RUnlock()

	pageName := agentDefPrefix + id

	var content string
	err := ws.store.db.QueryRow(
		"SELECT content FROM wiki WHERE name = ?",
		pageName,
	).Scan(&content)

	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("failed to get agent definition: %w", err)
	}

	var agent types.AgentDefinition
	if err := yaml.Unmarshal([]byte(content), &agent); err != nil {
		return nil, fmt.Errorf("failed to parse agent definition: %w", err)
	}

	return &agent, nil
}

// ListAgentDefinitions returns all agent definitions.
func (ws *WikiStore) ListAgentDefinitions() ([]*types.AgentDefinition, error) {
	ws.store.mu.RLock()
	defer ws.store.mu.RUnlock()

	rows, err := ws.store.db.Query(
		"SELECT content FROM wiki WHERE name LIKE ?",
		agentDefPrefix+"%",
	)
	if err != nil {
		return nil, fmt.Errorf("failed to list agent definitions: %w", err)
	}
	defer rows.Close()

	var agents []*types.AgentDefinition
	for rows.Next() {
		var content string
		if err := rows.Scan(&content); err != nil {
			return nil, fmt.Errorf("failed to scan agent definition: %w", err)
		}

		var agent types.AgentDefinition
		if err := yaml.Unmarshal([]byte(content), &agent); err != nil {
			continue // Skip invalid entries
		}
		agents = append(agents, &agent)
	}

	return agents, nil
}

// DeleteAgentDefinition removes an agent definition.
func (ws *WikiStore) DeleteAgentDefinition(id string) error {
	ws.store.mu.Lock()
	defer ws.store.mu.Unlock()

	pageName := agentDefPrefix + id

	result, err := ws.store.db.Exec("DELETE FROM wiki WHERE name = ?", pageName)
	if err != nil {
		return fmt.Errorf("failed to delete agent definition: %w", err)
	}

	rows, _ := result.RowsAffected()
	if rows == 0 {
		return fmt.Errorf("agent definition not found: %s", id)
	}

	return nil
}

// SaveWikiPage saves a general wiki page.
func (ws *WikiStore) SaveWikiPage(name string, content string, mimetype string) error {
	ws.store.mu.Lock()
	defer ws.store.mu.Unlock()

	if mimetype == "" {
		mimetype = "text/x-markdown"
	}

	now := float64(time.Now().Unix())

	_, err := ws.store.db.Exec(`
		INSERT INTO wiki (name, content, mimetype, mtime, ctime)
		VALUES (?, ?, ?, ?, ?)
		ON CONFLICT(name) DO UPDATE SET
			content = excluded.content,
			mimetype = excluded.mimetype,
			mtime = excluded.mtime
	`, name, content, mimetype, now, now)

	if err != nil {
		return fmt.Errorf("failed to save wiki page: %w", err)
	}

	return nil
}

// GetWikiPage retrieves a wiki page by name.
func (ws *WikiStore) GetWikiPage(name string) (string, string, error) {
	ws.store.mu.RLock()
	defer ws.store.mu.RUnlock()

	var content, mimetype string
	err := ws.store.db.QueryRow(
		"SELECT content, mimetype FROM wiki WHERE name = ?",
		name,
	).Scan(&content, &mimetype)

	if err == sql.ErrNoRows {
		return "", "", nil
	}
	if err != nil {
		return "", "", fmt.Errorf("failed to get wiki page: %w", err)
	}

	return content, mimetype, nil
}

// ListWikiPages returns all wiki page names.
func (ws *WikiStore) ListWikiPages() ([]string, error) {
	ws.store.mu.RLock()
	defer ws.store.mu.RUnlock()

	rows, err := ws.store.db.Query("SELECT name FROM wiki ORDER BY name")
	if err != nil {
		return nil, fmt.Errorf("failed to list wiki pages: %w", err)
	}
	defer rows.Close()

	var names []string
	for rows.Next() {
		var name string
		if err := rows.Scan(&name); err != nil {
			continue
		}
		names = append(names, name)
	}

	return names, nil
}

// SearchWikiPages searches wiki content.
func (ws *WikiStore) SearchWikiPages(query string) ([]string, error) {
	ws.store.mu.RLock()
	defer ws.store.mu.RUnlock()

	searchPattern := "%" + strings.ToLower(query) + "%"

	rows, err := ws.store.db.Query(
		"SELECT name FROM wiki WHERE LOWER(content) LIKE ? OR LOWER(name) LIKE ?",
		searchPattern, searchPattern,
	)
	if err != nil {
		return nil, fmt.Errorf("failed to search wiki: %w", err)
	}
	defer rows.Close()

	var names []string
	for rows.Next() {
		var name string
		if err := rows.Scan(&name); err != nil {
			continue
		}
		names = append(names, name)
	}

	return names, nil
}
