package crypto

import (
	"encoding/base64"
	"encoding/json"
	"fmt"

	"filippo.io/age"
	"github.com/roea-ai/roea/pkg/types"
)

// PayloadVersion is the current encrypted payload format version.
const PayloadVersion = 1

// PayloadService handles encryption and decryption of payload data.
type PayloadService struct {
	keyManager *KeyManager
}

// NewPayloadService creates a new PayloadService.
func NewPayloadService(keyManager *KeyManager) *PayloadService {
	return &PayloadService{
		keyManager: keyManager,
	}
}

// EncryptSecrets encrypts TaskSecrets into an EncryptedPayload.
func (ps *PayloadService) EncryptSecrets(secrets *types.TaskSecrets) (*types.EncryptedPayload, error) {
	return ps.EncryptSecretsTo(secrets, ps.keyManager.PublicKey())
}

// EncryptSecretsTo encrypts TaskSecrets to a specific recipient.
func (ps *PayloadService) EncryptSecretsTo(secrets *types.TaskSecrets, recipientKey string) (*types.EncryptedPayload, error) {
	plaintext, err := json.Marshal(secrets)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal secrets: %w", err)
	}

	ciphertext, err := EncryptToRecipient(plaintext, recipientKey)
	if err != nil {
		return nil, fmt.Errorf("failed to encrypt: %w", err)
	}

	return &types.EncryptedPayload{
		Version:    PayloadVersion,
		Recipient:  recipientKey[:12] + "...", // Public key hint
		Ciphertext: base64.StdEncoding.EncodeToString(ciphertext),
	}, nil
}

// DecryptSecrets decrypts an EncryptedPayload into TaskSecrets.
func (ps *PayloadService) DecryptSecrets(payload *types.EncryptedPayload) (*types.TaskSecrets, error) {
	return ps.DecryptSecretsWithIdentity(payload, ps.keyManager.Identity())
}

// DecryptSecretsWithIdentity decrypts using a specific identity.
func (ps *PayloadService) DecryptSecretsWithIdentity(payload *types.EncryptedPayload, identity *age.X25519Identity) (*types.TaskSecrets, error) {
	if payload == nil {
		return nil, fmt.Errorf("payload is nil")
	}

	ciphertext, err := base64.StdEncoding.DecodeString(payload.Ciphertext)
	if err != nil {
		return nil, fmt.Errorf("failed to decode ciphertext: %w", err)
	}

	plaintext, err := DecryptWithIdentity(ciphertext, identity)
	if err != nil {
		return nil, fmt.Errorf("failed to decrypt: %w", err)
	}

	var secrets types.TaskSecrets
	if err := json.Unmarshal(plaintext, &secrets); err != nil {
		return nil, fmt.Errorf("failed to unmarshal secrets: %w", err)
	}

	return &secrets, nil
}

// EncryptJSON encrypts any JSON-serializable data.
func (ps *PayloadService) EncryptJSON(data any) (*types.EncryptedPayload, error) {
	return ps.EncryptJSONTo(data, ps.keyManager.PublicKey())
}

// EncryptJSONTo encrypts JSON data to a specific recipient.
func (ps *PayloadService) EncryptJSONTo(data any, recipientKey string) (*types.EncryptedPayload, error) {
	plaintext, err := json.Marshal(data)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal data: %w", err)
	}

	ciphertext, err := EncryptToRecipient(plaintext, recipientKey)
	if err != nil {
		return nil, fmt.Errorf("failed to encrypt: %w", err)
	}

	return &types.EncryptedPayload{
		Version:    PayloadVersion,
		Recipient:  recipientKey[:12] + "...",
		Ciphertext: base64.StdEncoding.EncodeToString(ciphertext),
	}, nil
}

// DecryptJSON decrypts an EncryptedPayload into a target struct.
func (ps *PayloadService) DecryptJSON(payload *types.EncryptedPayload, target any) error {
	return ps.DecryptJSONWithIdentity(payload, ps.keyManager.Identity(), target)
}

// DecryptJSONWithIdentity decrypts using a specific identity.
func (ps *PayloadService) DecryptJSONWithIdentity(payload *types.EncryptedPayload, identity *age.X25519Identity, target any) error {
	if payload == nil {
		return fmt.Errorf("payload is nil")
	}

	ciphertext, err := base64.StdEncoding.DecodeString(payload.Ciphertext)
	if err != nil {
		return fmt.Errorf("failed to decode ciphertext: %w", err)
	}

	plaintext, err := DecryptWithIdentity(ciphertext, identity)
	if err != nil {
		return fmt.Errorf("failed to decrypt: %w", err)
	}

	if err := json.Unmarshal(plaintext, target); err != nil {
		return fmt.Errorf("failed to unmarshal: %w", err)
	}

	return nil
}

// EncryptRaw encrypts raw bytes.
func (ps *PayloadService) EncryptRaw(data []byte) (*types.EncryptedPayload, error) {
	ciphertext, err := EncryptToRecipient(data, ps.keyManager.PublicKey())
	if err != nil {
		return nil, fmt.Errorf("failed to encrypt: %w", err)
	}

	return &types.EncryptedPayload{
		Version:    PayloadVersion,
		Recipient:  ps.keyManager.PublicKeyHint(),
		Ciphertext: base64.StdEncoding.EncodeToString(ciphertext),
	}, nil
}

// DecryptRaw decrypts to raw bytes.
func (ps *PayloadService) DecryptRaw(payload *types.EncryptedPayload) ([]byte, error) {
	if payload == nil {
		return nil, fmt.Errorf("payload is nil")
	}

	ciphertext, err := base64.StdEncoding.DecodeString(payload.Ciphertext)
	if err != nil {
		return nil, fmt.Errorf("failed to decode ciphertext: %w", err)
	}

	return DecryptWithIdentity(ciphertext, ps.keyManager.Identity())
}
