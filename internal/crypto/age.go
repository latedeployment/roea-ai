package crypto

import (
	"bytes"
	"fmt"
	"io"

	"filippo.io/age"
)

// Encryptor handles age encryption operations.
type Encryptor struct {
	recipients []age.Recipient
}

// NewEncryptor creates an Encryptor for the given recipients.
func NewEncryptor(recipients ...age.Recipient) *Encryptor {
	return &Encryptor{
		recipients: recipients,
	}
}

// Encrypt encrypts data to all configured recipients.
func (e *Encryptor) Encrypt(plaintext []byte) ([]byte, error) {
	if len(e.recipients) == 0 {
		return nil, fmt.Errorf("no recipients configured")
	}

	var buf bytes.Buffer
	w, err := age.Encrypt(&buf, e.recipients...)
	if err != nil {
		return nil, fmt.Errorf("failed to create encryptor: %w", err)
	}

	if _, err := w.Write(plaintext); err != nil {
		return nil, fmt.Errorf("failed to write plaintext: %w", err)
	}

	if err := w.Close(); err != nil {
		return nil, fmt.Errorf("failed to close encryptor: %w", err)
	}

	return buf.Bytes(), nil
}

// Decryptor handles age decryption operations.
type Decryptor struct {
	identities []age.Identity
}

// NewDecryptor creates a Decryptor with the given identities.
func NewDecryptor(identities ...age.Identity) *Decryptor {
	return &Decryptor{
		identities: identities,
	}
}

// Decrypt decrypts data using configured identities.
func (d *Decryptor) Decrypt(ciphertext []byte) ([]byte, error) {
	if len(d.identities) == 0 {
		return nil, fmt.Errorf("no identities configured")
	}

	r, err := age.Decrypt(bytes.NewReader(ciphertext), d.identities...)
	if err != nil {
		return nil, fmt.Errorf("failed to decrypt: %w", err)
	}

	plaintext, err := io.ReadAll(r)
	if err != nil {
		return nil, fmt.Errorf("failed to read decrypted data: %w", err)
	}

	return plaintext, nil
}

// EncryptToRecipient encrypts data to a single recipient public key.
func EncryptToRecipient(plaintext []byte, publicKey string) ([]byte, error) {
	recipient, err := age.ParseX25519Recipient(publicKey)
	if err != nil {
		return nil, fmt.Errorf("failed to parse recipient: %w", err)
	}

	return NewEncryptor(recipient).Encrypt(plaintext)
}

// DecryptWithIdentity decrypts data using a single identity.
func DecryptWithIdentity(ciphertext []byte, identity *age.X25519Identity) ([]byte, error) {
	return NewDecryptor(identity).Decrypt(ciphertext)
}

// EncryptToMultiple encrypts data to multiple recipients.
func EncryptToMultiple(plaintext []byte, publicKeys []string) ([]byte, error) {
	recipients := make([]age.Recipient, 0, len(publicKeys))
	for _, pk := range publicKeys {
		recipient, err := age.ParseX25519Recipient(pk)
		if err != nil {
			return nil, fmt.Errorf("failed to parse recipient %s: %w", pk, err)
		}
		recipients = append(recipients, recipient)
	}

	return NewEncryptor(recipients...).Encrypt(plaintext)
}
