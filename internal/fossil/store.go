// Package fossil provides integration with Fossil SCM for data storage.
package fossil

import (
	"database/sql"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sync"

	_ "github.com/mattn/go-sqlite3"
)

// Store provides access to a Fossil repository.
type Store struct {
	fossilPath string // Path to .fossil file
	workDir    string // Working directory (.roea)
	db         *sql.DB
	mu         sync.RWMutex
}

// NewStore creates a new Store for the given fossil file.
func NewStore(fossilPath string) *Store {
	dir := filepath.Dir(fossilPath)
	return &Store{
		fossilPath: fossilPath,
		workDir:    filepath.Join(dir, ".roea"),
	}
}

// Initialize sets up the Fossil repository if it doesn't exist.
func (s *Store) Initialize() error {
	s.mu.Lock()
	defer s.mu.Unlock()

	// Create fossil repo if it doesn't exist
	if _, err := os.Stat(s.fossilPath); os.IsNotExist(err) {
		if err := s.createRepo(); err != nil {
			return fmt.Errorf("failed to create repo: %w", err)
		}
	}

	// Open the fossil repo
	if err := s.openRepo(); err != nil {
		return fmt.Errorf("failed to open repo: %w", err)
	}

	// Initialize custom ticket schema
	if err := s.initTicketSchema(); err != nil {
		return fmt.Errorf("failed to init ticket schema: %w", err)
	}

	return nil
}

// createRepo creates a new Fossil repository.
func (s *Store) createRepo() error {
	// Ensure directory exists
	dir := filepath.Dir(s.fossilPath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("failed to create directory: %w", err)
	}

	// Create new fossil repo
	cmd := exec.Command("fossil", "new", s.fossilPath)
	if output, err := cmd.CombinedOutput(); err != nil {
		return fmt.Errorf("fossil new failed: %s: %w", string(output), err)
	}

	return nil
}

// openRepo opens the Fossil repository.
func (s *Store) openRepo() error {
	// Ensure work directory exists
	if err := os.MkdirAll(s.workDir, 0755); err != nil {
		return fmt.Errorf("failed to create work dir: %w", err)
	}

	// Check if already open
	checkCmd := exec.Command("fossil", "status")
	checkCmd.Dir = s.workDir
	if checkCmd.Run() == nil {
		// Already open, just connect to DB
		return s.connectDB()
	}

	// Open the fossil repo
	cmd := exec.Command("fossil", "open", s.fossilPath, "--workdir", s.workDir)
	cmd.Dir = s.workDir
	if output, err := cmd.CombinedOutput(); err != nil {
		// Try to continue if already open
		if exitErr, ok := err.(*exec.ExitError); ok && exitErr.ExitCode() == 1 {
			// May already be open, try to connect
			return s.connectDB()
		}
		return fmt.Errorf("fossil open failed: %s: %w", string(output), err)
	}

	return s.connectDB()
}

// connectDB opens a direct SQLite connection to the fossil database.
func (s *Store) connectDB() error {
	db, err := sql.Open("sqlite3", s.fossilPath+"?mode=rw")
	if err != nil {
		return fmt.Errorf("failed to open database: %w", err)
	}

	// Test connection
	if err := db.Ping(); err != nil {
		db.Close()
		return fmt.Errorf("failed to ping database: %w", err)
	}

	s.db = db
	return nil
}

// initTicketSchema adds custom fields for Roea tasks.
func (s *Store) initTicketSchema() error {
	// Check if custom fields already exist
	var count int
	err := s.db.QueryRow(`
		SELECT COUNT(*) FROM pragma_table_info('ticket')
		WHERE name = 'agent_type'
	`).Scan(&count)

	if err != nil {
		// Table might not exist yet, create basic ticket table
		return s.createTicketTable()
	}

	if count > 0 {
		// Already initialized
		return nil
	}

	// Add custom columns
	alterStatements := []string{
		"ALTER TABLE ticket ADD COLUMN agent_type TEXT",
		"ALTER TABLE ticket ADD COLUMN execution_mode TEXT",
		"ALTER TABLE ticket ADD COLUMN model TEXT",
		"ALTER TABLE ticket ADD COLUMN repo_url TEXT",
		"ALTER TABLE ticket ADD COLUMN branch TEXT",
		"ALTER TABLE ticket ADD COLUMN worktree TEXT",
		"ALTER TABLE ticket ADD COLUMN secrets_encrypted TEXT",
		"ALTER TABLE ticket ADD COLUMN started_at TEXT",
		"ALTER TABLE ticket ADD COLUMN completed_at TEXT",
		"ALTER TABLE ticket ADD COLUMN result TEXT",
		"ALTER TABLE ticket ADD COLUMN error_message TEXT",
		"ALTER TABLE ticket ADD COLUMN parent_id TEXT",
		"ALTER TABLE ticket ADD COLUMN labels TEXT",
	}

	for _, stmt := range alterStatements {
		if _, err := s.db.Exec(stmt); err != nil {
			// Ignore errors for columns that already exist
			continue
		}
	}

	return nil
}

// createTicketTable creates the ticket table if it doesn't exist.
func (s *Store) createTicketTable() error {
	_, err := s.db.Exec(`
		CREATE TABLE IF NOT EXISTS ticket (
			tkt_id INTEGER PRIMARY KEY,
			tkt_uuid TEXT UNIQUE NOT NULL,
			type TEXT,
			status TEXT DEFAULT 'pending',
			subsystem TEXT,
			priority INTEGER DEFAULT 5,
			severity INTEGER DEFAULT 5,
			foundin TEXT,
			private_contact TEXT,
			resolution TEXT,
			title TEXT,
			comment TEXT,
			tkt_mtime REAL,
			tkt_ctime REAL,
			agent_type TEXT,
			execution_mode TEXT,
			model TEXT,
			repo_url TEXT,
			branch TEXT,
			worktree TEXT,
			secrets_encrypted TEXT,
			started_at TEXT,
			completed_at TEXT,
			result TEXT,
			error_message TEXT,
			parent_id TEXT,
			labels TEXT
		)
	`)
	return err
}

// Close closes the store.
func (s *Store) Close() error {
	s.mu.Lock()
	defer s.mu.Unlock()

	if s.db != nil {
		if err := s.db.Close(); err != nil {
			return err
		}
		s.db = nil
	}

	return nil
}

// DB returns the underlying database connection.
func (s *Store) DB() *sql.DB {
	return s.db
}

// FossilPath returns the path to the fossil file.
func (s *Store) FossilPath() string {
	return s.fossilPath
}

// WorkDir returns the working directory path.
func (s *Store) WorkDir() string {
	return s.workDir
}

// RunFossilCommand executes a fossil command in the work directory.
func (s *Store) RunFossilCommand(args ...string) ([]byte, error) {
	cmd := exec.Command("fossil", args...)
	cmd.Dir = s.workDir
	return cmd.CombinedOutput()
}

// Sync synchronizes with a remote Fossil server.
func (s *Store) Sync(remoteURL string) error {
	args := []string{"sync"}
	if remoteURL != "" {
		args = append(args, remoteURL)
	}

	output, err := s.RunFossilCommand(args...)
	if err != nil {
		return fmt.Errorf("fossil sync failed: %s: %w", string(output), err)
	}

	return nil
}
