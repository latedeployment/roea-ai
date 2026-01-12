// Package crypto provides age encryption functionality for Roea.
package crypto

import (
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"filippo.io/age"
)

// KeyManager handles age key generation and storage.
type KeyManager struct {
	identityPath string
	identity     *age.X25519Identity
	publicKey    string
}

// NewKeyManager creates a new KeyManager with the specified identity path.
func NewKeyManager(identityPath string) *KeyManager {
	return &KeyManager{
		identityPath: identityPath,
	}
}

// Initialize loads an existing identity or generates a new one.
func (km *KeyManager) Initialize() error {
	if km.fileExists(km.identityPath) {
		return km.loadIdentity()
	}
	return km.generateIdentity()
}

// fileExists checks if a file exists at the given path.
func (km *KeyManager) fileExists(path string) bool {
	_, err := os.Stat(path)
	return err == nil
}

// generateIdentity creates a new age X25519 identity.
func (km *KeyManager) generateIdentity() error {
	identity, err := age.GenerateX25519Identity()
	if err != nil {
		return fmt.Errorf("failed to generate identity: %w", err)
	}

	// Ensure directory exists
	dir := filepath.Dir(km.identityPath)
	if err := os.MkdirAll(dir, 0700); err != nil {
		return fmt.Errorf("failed to create directory: %w", err)
	}

	// Write identity to file with restricted permissions
	content := fmt.Sprintf("# created: roea-ai\n# public key: %s\n%s\n",
		identity.Recipient().String(),
		identity.String(),
	)

	if err := os.WriteFile(km.identityPath, []byte(content), 0600); err != nil {
		return fmt.Errorf("failed to write identity file: %w", err)
	}

	km.identity = identity
	km.publicKey = identity.Recipient().String()

	return nil
}

// loadIdentity reads an existing identity from disk.
func (km *KeyManager) loadIdentity() error {
	data, err := os.ReadFile(km.identityPath)
	if err != nil {
		return fmt.Errorf("failed to read identity file: %w", err)
	}

	// Parse the identity (skip comment lines)
	lines := strings.Split(string(data), "\n")
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}

		identity, err := age.ParseX25519Identity(line)
		if err != nil {
			return fmt.Errorf("failed to parse identity: %w", err)
		}

		km.identity = identity
		km.publicKey = identity.Recipient().String()
		return nil
	}

	return fmt.Errorf("no identity found in file")
}

// PublicKey returns the public key string.
func (km *KeyManager) PublicKey() string {
	return km.publicKey
}

// PublicKeyHint returns a shortened version of the public key for identification.
func (km *KeyManager) PublicKeyHint() string {
	if len(km.publicKey) > 12 {
		return km.publicKey[:12] + "..."
	}
	return km.publicKey
}

// Identity returns the underlying age identity.
func (km *KeyManager) Identity() *age.X25519Identity {
	return km.identity
}

// Recipient returns the age recipient for encryption.
func (km *KeyManager) Recipient() *age.X25519Recipient {
	return km.identity.Recipient()
}

// ParseRecipient parses a public key string into a recipient.
func ParseRecipient(publicKey string) (*age.X25519Recipient, error) {
	recipient, err := age.ParseX25519Recipient(publicKey)
	if err != nil {
		return nil, fmt.Errorf("failed to parse recipient: %w", err)
	}
	return recipient, nil
}

// GenerateEphemeralIdentity creates a new temporary identity for agent use.
func GenerateEphemeralIdentity() (*age.X25519Identity, error) {
	return age.GenerateX25519Identity()
}

// ExportIdentity returns the identity as a string for serialization.
func ExportIdentity(identity *age.X25519Identity) string {
	var buf bytes.Buffer
	buf.WriteString(fmt.Sprintf("# public key: %s\n", identity.Recipient().String()))
	buf.WriteString(identity.String())
	buf.WriteString("\n")
	return buf.String()
}

// ImportIdentity parses an identity from a string.
func ImportIdentity(data string) (*age.X25519Identity, error) {
	lines := strings.Split(data, "\n")
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		return age.ParseX25519Identity(line)
	}
	return nil, fmt.Errorf("no identity found")
}
