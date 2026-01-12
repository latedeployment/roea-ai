package types

// Config represents the main configuration for Roea.
type Config struct {
	Server    ServerConfig    `yaml:"server"`
	Fossil    FossilConfig    `yaml:"fossil"`
	Crypto    CryptoConfig    `yaml:"crypto"`
	Executors ExecutorsConfig `yaml:"executors"`
	Git       GitConfig       `yaml:"git"`
	MCP       MCPConfig       `yaml:"mcp"`
	Models    ModelsConfig    `yaml:"models"`
}

// ServerConfig defines HTTP server settings.
type ServerConfig struct {
	Host string `yaml:"host"`
	Port int    `yaml:"port"`
}

// FossilConfig defines Fossil SCM settings.
type FossilConfig struct {
	Path     string `yaml:"path"`      // Path to .fossil file
	AutoSync bool   `yaml:"auto_sync"` // Enable for distributed setup
	SyncURL  string `yaml:"sync_url"`  // Remote Fossil server URL
}

// CryptoConfig defines encryption settings.
type CryptoConfig struct {
	IdentityPath string `yaml:"identity_path"` // Path to age identity file
}

// ExecutorsConfig defines execution backend settings.
type ExecutorsConfig struct {
	Local LocalExecutorConfig `yaml:"local"`
	K8s   K8sExecutorConfig   `yaml:"k8s"`
	VM    VMExecutorConfig    `yaml:"vm"`
}

// LocalExecutorConfig defines local subprocess executor settings.
type LocalExecutorConfig struct {
	Enabled       bool   `yaml:"enabled"`
	MaxConcurrent int    `yaml:"max_concurrent"`
	WorktreeBase  string `yaml:"worktree_base"` // Base directory for worktrees
}

// K8sExecutorConfig defines Kubernetes executor settings.
type K8sExecutorConfig struct {
	Enabled   bool   `yaml:"enabled"`
	Namespace string `yaml:"namespace"`
	Image     string `yaml:"image"` // Agent runtime container image
}

// VMExecutorConfig defines VM executor settings.
type VMExecutorConfig struct {
	Enabled   bool   `yaml:"enabled"`
	QEMUPath  string `yaml:"qemu_path"`
	BaseImage string `yaml:"base_image"` // Path to base VM image
}

// GitConfig defines Git operation settings.
type GitConfig struct {
	DefaultRemote string `yaml:"default_remote"`
	BranchPrefix  string `yaml:"branch_prefix"`
	AutoPush      bool   `yaml:"auto_push"`
}

// MCPConfig defines MCP server settings.
type MCPConfig struct {
	Enabled bool `yaml:"enabled"`
}

// ModelsConfig defines model routing settings.
type ModelsConfig struct {
	Default   string                     `yaml:"default"`
	Providers map[string]ProviderConfig  `yaml:"providers"`
}

// ProviderConfig defines settings for an AI provider.
type ProviderConfig struct {
	APIKeyEncrypted string `yaml:"api_key_encrypted"` // age-encrypted API key
}

// DefaultConfig returns a configuration with sensible defaults.
func DefaultConfig() *Config {
	return &Config{
		Server: ServerConfig{
			Host: "0.0.0.0",
			Port: 8080,
		},
		Fossil: FossilConfig{
			Path:     "./roea.fossil",
			AutoSync: false,
		},
		Crypto: CryptoConfig{
			IdentityPath: "./roea.key",
		},
		Executors: ExecutorsConfig{
			Local: LocalExecutorConfig{
				Enabled:       true,
				MaxConcurrent: 4,
				WorktreeBase:  "/tmp/roea-worktrees",
			},
			K8s: K8sExecutorConfig{
				Enabled:   false,
				Namespace: "roea-agents",
			},
			VM: VMExecutorConfig{
				Enabled: false,
			},
		},
		Git: GitConfig{
			DefaultRemote: "origin",
			BranchPrefix:  "roea/",
			AutoPush:      true,
		},
		MCP: MCPConfig{
			Enabled: true,
		},
		Models: ModelsConfig{
			Default: "claude-sonnet-4-20250514",
		},
	}
}
