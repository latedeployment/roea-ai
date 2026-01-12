package types

// EncryptedPayload contains age-encrypted data.
type EncryptedPayload struct {
	Version    int    `json:"v"`          // Payload format version
	Recipient  string `json:"r"`          // age public key hint
	Ciphertext string `json:"c"`          // base64 age ciphertext
}

// TaskSecrets contains decrypted secrets for a task.
type TaskSecrets struct {
	APIKeys     map[string]string `json:"api_keys"`
	Credentials map[string]string `json:"credentials"`
	Tokens      map[string]string `json:"tokens"`
	Custom      map[string]string `json:"custom"`
}

// SecureTaskContext wraps task data with encrypted secrets.
type SecureTaskContext struct {
	TaskID     string            `json:"task_id"`
	PublicData map[string]any    `json:"public"`
	Secrets    *EncryptedPayload `json:"secrets,omitempty"`
}
