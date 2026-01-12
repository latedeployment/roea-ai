// Package models provides model routing and provider management.
package models

import (
	"fmt"
	"strings"
	"sync"

	"github.com/roea-ai/roea/internal/crypto"
	"github.com/roea-ai/roea/pkg/types"
)

// Router manages model selection and provider configuration.
type Router struct {
	config         *types.ModelsConfig
	payloadService *crypto.PayloadService

	// Cached decrypted API keys
	apiKeysMu sync.RWMutex
	apiKeys   map[string]string

	// Model to provider mapping
	modelProviders map[string]string
}

// Provider represents an AI model provider.
type Provider struct {
	Name   string
	Models []string
	APIKey string
}

// NewRouter creates a new model Router.
func NewRouter(config *types.ModelsConfig, payloadService *crypto.PayloadService) *Router {
	r := &Router{
		config:         config,
		payloadService: payloadService,
		apiKeys:        make(map[string]string),
		modelProviders: make(map[string]string),
	}

	r.initModelProviders()
	return r
}

// initModelProviders sets up the model to provider mappings.
func (r *Router) initModelProviders() {
	// Anthropic models
	anthropicModels := []string{
		"claude-opus-4-20250514",
		"claude-sonnet-4-20250514",
		"claude-3-opus-20240229",
		"claude-3-5-sonnet-20241022",
		"claude-3-sonnet-20240229",
		"claude-3-haiku-20240307",
	}
	for _, model := range anthropicModels {
		r.modelProviders[model] = "anthropic"
	}

	// OpenAI models
	openaiModels := []string{
		"gpt-4o",
		"gpt-4o-mini",
		"gpt-4-turbo",
		"gpt-4",
		"gpt-3.5-turbo",
		"o1-preview",
		"o1-mini",
	}
	for _, model := range openaiModels {
		r.modelProviders[model] = "openai"
	}
}

// GetDefaultModel returns the default model.
func (r *Router) GetDefaultModel() string {
	if r.config.Default != "" {
		return r.config.Default
	}
	return "claude-sonnet-4-20250514"
}

// GetProviderForModel returns the provider for a given model.
func (r *Router) GetProviderForModel(model string) string {
	if provider, ok := r.modelProviders[model]; ok {
		return provider
	}

	// Try to infer from model name
	if strings.HasPrefix(model, "claude") {
		return "anthropic"
	}
	if strings.HasPrefix(model, "gpt") || strings.HasPrefix(model, "o1") {
		return "openai"
	}

	return "unknown"
}

// GetAPIKey returns the API key for a provider.
func (r *Router) GetAPIKey(provider string) (string, error) {
	// Check cache first
	r.apiKeysMu.RLock()
	if key, ok := r.apiKeys[provider]; ok {
		r.apiKeysMu.RUnlock()
		return key, nil
	}
	r.apiKeysMu.RUnlock()

	// Get encrypted key from config
	providerConfig, ok := r.config.Providers[provider]
	if !ok {
		return "", fmt.Errorf("provider not configured: %s", provider)
	}

	if providerConfig.APIKeyEncrypted == "" {
		return "", fmt.Errorf("no API key configured for provider: %s", provider)
	}

	// Decrypt the key
	if r.payloadService == nil {
		return "", fmt.Errorf("payload service not configured")
	}

	// Parse the encrypted payload
	payload := &types.EncryptedPayload{
		Version:    1,
		Ciphertext: providerConfig.APIKeyEncrypted,
	}

	var decrypted struct {
		Key string `json:"key"`
	}

	if err := r.payloadService.DecryptJSON(payload, &decrypted); err != nil {
		return "", fmt.Errorf("failed to decrypt API key: %w", err)
	}

	// Cache the decrypted key
	r.apiKeysMu.Lock()
	r.apiKeys[provider] = decrypted.Key
	r.apiKeysMu.Unlock()

	return decrypted.Key, nil
}

// SetAPIKey encrypts and stores an API key for a provider.
func (r *Router) SetAPIKey(provider string, apiKey string) error {
	if r.payloadService == nil {
		return fmt.Errorf("payload service not configured")
	}

	// Encrypt the key
	data := struct {
		Key string `json:"key"`
	}{Key: apiKey}

	payload, err := r.payloadService.EncryptJSON(data)
	if err != nil {
		return fmt.Errorf("failed to encrypt API key: %w", err)
	}

	// Update config
	if r.config.Providers == nil {
		r.config.Providers = make(map[string]types.ProviderConfig)
	}
	r.config.Providers[provider] = types.ProviderConfig{
		APIKeyEncrypted: payload.Ciphertext,
	}

	// Update cache
	r.apiKeysMu.Lock()
	r.apiKeys[provider] = apiKey
	r.apiKeysMu.Unlock()

	return nil
}

// SelectModel chooses the appropriate model for a task.
func (r *Router) SelectModel(taskModel string, agentModel string) string {
	// Priority: task model > agent model > default
	if taskModel != "" {
		return taskModel
	}
	if agentModel != "" {
		return agentModel
	}
	return r.GetDefaultModel()
}

// ListProviders returns all configured providers.
func (r *Router) ListProviders() []string {
	providers := make([]string, 0, len(r.config.Providers))
	for name := range r.config.Providers {
		providers = append(providers, name)
	}
	return providers
}

// ListModels returns all known models for a provider.
func (r *Router) ListModels(provider string) []string {
	models := make([]string, 0)
	for model, p := range r.modelProviders {
		if p == provider {
			models = append(models, model)
		}
	}
	return models
}

// ValidateModel checks if a model is valid.
func (r *Router) ValidateModel(model string) bool {
	_, ok := r.modelProviders[model]
	return ok
}

// GetModelInfo returns information about a model.
func (r *Router) GetModelInfo(model string) *ModelInfo {
	provider := r.GetProviderForModel(model)
	if provider == "unknown" {
		return nil
	}

	return &ModelInfo{
		ID:       model,
		Provider: provider,
	}
}

// ModelInfo contains information about a model.
type ModelInfo struct {
	ID       string `json:"id"`
	Provider string `json:"provider"`
}

// ClearCache clears the cached API keys.
func (r *Router) ClearCache() {
	r.apiKeysMu.Lock()
	r.apiKeys = make(map[string]string)
	r.apiKeysMu.Unlock()
}
