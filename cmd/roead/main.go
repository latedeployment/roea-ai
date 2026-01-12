// Package main is the entry point for the Roea daemon.
package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"path/filepath"
	"syscall"
	"time"

	"gopkg.in/yaml.v3"

	"github.com/roea-ai/roea/internal/api"
	"github.com/roea-ai/roea/internal/core/agent"
	"github.com/roea-ai/roea/internal/core/execution"
	"github.com/roea-ai/roea/internal/core/git"
	"github.com/roea-ai/roea/internal/core/task"
	"github.com/roea-ai/roea/internal/crypto"
	localexec "github.com/roea-ai/roea/internal/executor/local"
	"github.com/roea-ai/roea/internal/fossil"
	"github.com/roea-ai/roea/internal/mcp"
	"github.com/roea-ai/roea/internal/models"
	"github.com/roea-ai/roea/pkg/types"
)

var (
	configPath   = flag.String("config", "", "Path to config file")
	initMode     = flag.Bool("init", false, "Initialize a new Roea instance")
	projectPath  = flag.String("path", ".", "Project path for initialization")
	showVersion  = flag.Bool("version", false, "Show version")
)

const version = "0.1.0"

func main() {
	flag.Parse()

	if *showVersion {
		fmt.Printf("roead version %s\n", version)
		os.Exit(0)
	}

	if *initMode {
		if err := initializeRoea(*projectPath); err != nil {
			log.Fatalf("Initialization failed: %v", err)
		}
		fmt.Println("Roea initialized successfully!")
		os.Exit(0)
	}

	// Load configuration
	config, err := loadConfig(*configPath)
	if err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	// Run the server
	if err := run(config); err != nil {
		log.Fatalf("Server error: %v", err)
	}
}

func loadConfig(path string) (*types.Config, error) {
	// Use default config if no path specified
	if path == "" {
		// Try common paths
		candidates := []string{
			"roea.yaml",
			"roea.yml",
			".roea/config.yaml",
		}
		for _, c := range candidates {
			if _, err := os.Stat(c); err == nil {
				path = c
				break
			}
		}
	}

	// Return default config if no file found
	if path == "" {
		return types.DefaultConfig(), nil
	}

	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read config: %w", err)
	}

	config := types.DefaultConfig()
	if err := yaml.Unmarshal(data, config); err != nil {
		return nil, fmt.Errorf("failed to parse config: %w", err)
	}

	return config, nil
}

func run(config *types.Config) error {
	log.Printf("Starting Roea daemon v%s", version)

	// Initialize crypto
	keyManager := crypto.NewKeyManager(config.Crypto.IdentityPath)
	if err := keyManager.Initialize(); err != nil {
		return fmt.Errorf("failed to initialize crypto: %w", err)
	}
	log.Printf("Crypto initialized, public key: %s", keyManager.PublicKeyHint())

	payloadService := crypto.NewPayloadService(keyManager)

	// Initialize Fossil store
	store := fossil.NewStore(config.Fossil.Path)
	if err := store.Initialize(); err != nil {
		return fmt.Errorf("failed to initialize Fossil store: %w", err)
	}
	defer store.Close()
	log.Printf("Fossil store initialized: %s", config.Fossil.Path)

	// Initialize sub-stores
	ticketStore := fossil.NewTicketStore(store)
	wikiStore := fossil.NewWikiStore(store)
	if err := wikiStore.Initialize(); err != nil {
		return fmt.Errorf("failed to initialize wiki store: %w", err)
	}
	artifactStore := fossil.NewArtifactStore(store)
	if err := artifactStore.Initialize(); err != nil {
		return fmt.Errorf("failed to initialize artifact store: %w", err)
	}

	// Initialize core components
	taskManager := task.NewManager(ticketStore, artifactStore, payloadService)
	agentPool := agent.NewPool(wikiStore)
	gitManager := git.NewManager(&config.Git, config.Executors.Local.WorktreeBase)
	modelRouter := models.NewRouter(&config.Models, payloadService)

	// Initialize execution engine
	executionEngine := execution.NewEngine(taskManager, agentPool)

	// Register executors
	if config.Executors.Local.Enabled {
		localExecutor := localexec.NewExecutor(&config.Executors.Local)
		executionEngine.RegisterExecutor(localExecutor)
		log.Printf("Local executor enabled (max concurrent: %d)", config.Executors.Local.MaxConcurrent)
	}

	// Initialize MCP server
	mcpServer := mcp.NewServer(taskManager, artifactStore, payloadService)

	// Initialize API router
	router := api.NewRouter(taskManager, agentPool, executionEngine, gitManager, mcpServer)

	// Create HTTP server
	addr := fmt.Sprintf("%s:%d", config.Server.Host, config.Server.Port)
	server := &http.Server{
		Addr:    addr,
		Handler: router.Handler(),
	}

	// Start server in goroutine
	go func() {
		log.Printf("Server listening on %s", addr)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("Server error: %v", err)
		}
	}()

	// Print startup info
	log.Printf("Roea AI orchestrator ready!")
	log.Printf("  API: http://%s/api/v1", addr)
	log.Printf("  MCP: http://%s/api/v1/mcp", addr)
	log.Printf("  WebSocket: ws://%s/ws", addr)
	log.Printf("  Default model: %s", modelRouter.GetDefaultModel())

	// Wait for shutdown signal
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit

	log.Println("Shutting down server...")

	// Graceful shutdown
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	if err := server.Shutdown(ctx); err != nil {
		return fmt.Errorf("server shutdown failed: %w", err)
	}

	log.Println("Server stopped")
	return nil
}

func initializeRoea(projectPath string) error {
	absPath, err := filepath.Abs(projectPath)
	if err != nil {
		return err
	}

	// Create .roea directory
	roeaDir := filepath.Join(absPath, ".roea")
	if err := os.MkdirAll(roeaDir, 0755); err != nil {
		return fmt.Errorf("failed to create .roea directory: %w", err)
	}

	// Create default config
	config := types.DefaultConfig()
	config.Fossil.Path = filepath.Join(absPath, "roea.fossil")
	config.Crypto.IdentityPath = filepath.Join(roeaDir, "roea.key")
	config.Executors.Local.WorktreeBase = filepath.Join(absPath, ".roea", "worktrees")

	configData, err := yaml.Marshal(config)
	if err != nil {
		return fmt.Errorf("failed to marshal config: %w", err)
	}

	configPath := filepath.Join(absPath, "roea.yaml")
	if err := os.WriteFile(configPath, configData, 0644); err != nil {
		return fmt.Errorf("failed to write config: %w", err)
	}
	fmt.Printf("Created config: %s\n", configPath)

	// Initialize crypto
	keyManager := crypto.NewKeyManager(config.Crypto.IdentityPath)
	if err := keyManager.Initialize(); err != nil {
		return fmt.Errorf("failed to initialize crypto: %w", err)
	}
	fmt.Printf("Created identity: %s\n", config.Crypto.IdentityPath)
	fmt.Printf("Public key: %s\n", keyManager.PublicKey())

	// Initialize Fossil store
	store := fossil.NewStore(config.Fossil.Path)
	if err := store.Initialize(); err != nil {
		return fmt.Errorf("failed to initialize Fossil store: %w", err)
	}
	store.Close()
	fmt.Printf("Created Fossil repo: %s\n", config.Fossil.Path)

	// Initialize wiki store for agent definitions
	store = fossil.NewStore(config.Fossil.Path)
	if err := store.Initialize(); err != nil {
		return fmt.Errorf("failed to reopen Fossil store: %w", err)
	}
	wikiStore := fossil.NewWikiStore(store)
	if err := wikiStore.Initialize(); err != nil {
		return fmt.Errorf("failed to initialize wiki store: %w", err)
	}

	// Initialize artifact store
	artifactStore := fossil.NewArtifactStore(store)
	if err := artifactStore.Initialize(); err != nil {
		return fmt.Errorf("failed to initialize artifact store: %w", err)
	}

	store.Close()

	// Create worktrees directory
	if err := os.MkdirAll(config.Executors.Local.WorktreeBase, 0755); err != nil {
		return fmt.Errorf("failed to create worktrees directory: %w", err)
	}
	fmt.Printf("Created worktrees dir: %s\n", config.Executors.Local.WorktreeBase)

	fmt.Println("\nRoea initialization complete!")
	fmt.Println("Run 'roead' to start the server.")

	return nil
}
