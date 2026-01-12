# Roea AI - Architecture Plan

**Roea AI** (רועה AI - "AI Herder" in Hebrew) is an AI agent orchestrator that manages, spawns, and coordinates multiple AI coding agents.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Web UI (Next.js)                                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Kanban Board│  │ Agent Mgmt  │  │ Repo Config │  │ Execution Monitor   │ │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      │ REST/WebSocket/gRPC
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Roea Core (Go Backend)                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ Task Manager │  │ Agent Pool   │  │ Execution Eng│  │ MCP Server      │  │
│  │ (Fossil SCM) │  │              │  │              │  │ (reports/ctrl)  │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  └─────────────────┘  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ Git Manager  │  │ Model Router │  │ Fossil Store │  │ K8s Controller  │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  └─────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                    Age Encryption Layer                              │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    ▼                 ▼                 ▼
            ┌─────────────┐   ┌─────────────┐   ┌─────────────┐
            │ Local Exec  │   │ K8s Exec    │   │ VM Exec     │
            │ (subprocess)│   │ (pods/jobs) │   │ (QEMU)      │
            └─────────────┘   └─────────────┘   └─────────────┘
                    │                 │                 │
                    ▼                 ▼                 ▼
            ┌─────────────────────────────────────────────────┐
            │              Agent Runtimes                      │
            │  ┌───────────┐ ┌───────────┐ ┌───────────────┐  │
            │  │Claude Code│ │  OpenCode │ │    Codex      │  │
            │  └───────────┘ └───────────┘ └───────────────┘  │
            └─────────────────────────────────────────────────┘
```

---

## Why Fossil SCM for Storage?

Fossil SCM is a perfect fit for Roea AI:

1. **Single SQLite file** - The entire state (tasks, wiki, attachments) is one `.fossil` file
2. **Built-in ticketing** - Maps directly to our task/kanban concept
3. **Built-in wiki** - Agent documentation, runbooks, context
4. **Artifact storage** - Store task outputs, logs, attachments
5. **Sync capability** - Can sync Fossil repos between local and K8s instances
6. **Audit trail** - Every change is versioned
7. **No external DB** - No PostgreSQL/MySQL dependency
8. **Web UI included** - Fossil has a built-in web UI we can leverage or replace

### Fossil → Roea Mapping

| Fossil Concept | Roea Concept |
|----------------|--------------|
| Ticket | Task |
| Ticket status | Task status (pending/running/done) |
| Ticket type | Agent type assignment |
| Wiki page | Agent definition / documentation |
| Attachment | Task artifacts / logs |
| Technote | Execution reports |
| Tag | Labels |

---

## Age Encryption for Secrets

All sensitive data passed between components uses [age](https://age-encryption.org/):

```
┌─────────────┐                              ┌─────────────┐
│ Roea Server │                              │   Agent     │
│             │                              │  (K8s/VM)   │
│ age-keygen  │──── public key ────────────▶│             │
│ (identity)  │                              │             │
│             │◀─── encrypted secrets ───────│ age encrypt │
│ age decrypt │                              │             │
└─────────────┘                              └─────────────┘
```

### What Gets Encrypted

- API keys (Anthropic, OpenAI, GitHub tokens)
- Task secrets (passwords, credentials for specific tasks)
- Agent-to-server authentication tokens
- Sensitive task context

### Key Management

```bash
# Server generates identity (kept secure)
age-keygen -o roea-server.key
# Public key: age1xxxxxxx...

# Each agent gets the server's public key
# Agents encrypt secrets before sending
# Only server can decrypt with private key

# For K8s: identity mounted as secret
# For local: identity in config directory
```

### Encrypted Payload Format

```go
type EncryptedPayload struct {
    Recipient string `json:"recipient"` // age public key
    Ciphertext []byte `json:"ciphertext"` // age-encrypted data
    Nonce      string `json:"nonce"`
}

// Usage in MCP
type SecureTaskContext struct {
    TaskID     string           `json:"task_id"`
    PublicData map[string]any   `json:"public"`
    Secrets    EncryptedPayload `json:"secrets"` // age-encrypted
}
```

---

## Core Components

### 1. Roea Core (Go Backend)

#### 1.1 Task Manager (Fossil-backed)

Uses Fossil's ticket system as the backing store:

```go
type FossilTaskStore struct {
    repoPath string // path to .fossil file
}

func (s *FossilTaskStore) CreateTask(t *Task) error {
    // Creates a Fossil ticket
    // fossil ticket add ...
}

func (s *FossilTaskStore) UpdateTask(id string, updates map[string]any) error {
    // Updates ticket fields
    // fossil ticket change ...
}

func (s *FossilTaskStore) ListTasks(filter TaskFilter) ([]Task, error) {
    // SQL query against Fossil's SQLite
    // SELECT * FROM ticket WHERE ...
}
```

#### 1.2 Agent Pool
- Registry of available agent definitions
- Agent templates stored as Fossil wiki pages
- Configuration:
  - System prompts
  - Allowed tools/MCP servers
  - Model preferences
  - Resource limits

#### 1.3 Execution Engine
- Spawns agent processes based on execution mode
- Modes:
  - **Ralph Wiggum Loop**: Bash loop, agent picks tasks, outputs `END` to terminate
  - **Spec Flow**: Structured workflow with defined stages
  - **Single Shot**: One task, one execution
  - **Continuous**: Long-running agent monitoring queue

#### 1.4 MCP Server
- Agents report progress via MCP
- All secrets passed through age-encrypted payloads
- Tools exposed:
  - `roea_report_progress`
  - `roea_complete_task`
  - `roea_fail_task`
  - `roea_spawn_subtask`
  - `roea_get_secrets` (returns age-encrypted blob)

#### 1.5 Git Manager
- Clone/pull repositories
- Worktree management for parallel agent work
- Branch management per task
- Can optionally store worktrees in Fossil artifacts

#### 1.6 Model Router
- Model selection per agent/task
- Provider abstraction
- API keys stored age-encrypted in Fossil

---

## Directory Structure

```
roea-ai/
├── cmd/
│   └── roead/                  # Main daemon binary
│       └── main.go
├── internal/
│   ├── api/                    # REST/gRPC API handlers
│   │   ├── handlers/
│   │   ├── middleware/
│   │   └── router.go
│   ├── core/
│   │   ├── task/               # Task management
│   │   ├── agent/              # Agent pool & definitions
│   │   ├── execution/          # Execution engine
│   │   └── git/                # Git operations
│   ├── fossil/                 # Fossil SCM integration
│   │   ├── store.go            # SQLite access to .fossil
│   │   ├── tickets.go          # Ticket/task operations
│   │   ├── wiki.go             # Wiki/agent definitions
│   │   └── artifacts.go        # Attachment storage
│   ├── crypto/                 # Age encryption
│   │   ├── age.go              # Encrypt/decrypt
│   │   ├── keys.go             # Key management
│   │   └── payload.go          # Encrypted payload types
│   ├── mcp/                    # MCP server implementation
│   │   ├── server.go
│   │   ├── tools.go
│   │   └── transport.go
│   ├── executor/               # Execution backends
│   │   ├── local/
│   │   ├── k8s/
│   │   └── vm/
│   └── models/                 # Model router
├── pkg/                        # Public packages
│   ├── types/                  # Shared types
│   └── client/                 # Go client for API
├── web/                        # Next.js frontend
│   ├── src/
│   │   ├── app/
│   │   ├── components/
│   │   │   ├── kanban/
│   │   │   ├── agents/
│   │   │   └── monitor/
│   │   ├── lib/
│   │   └── hooks/
│   ├── package.json
│   └── next.config.js
├── deploy/
│   ├── docker/
│   │   ├── Dockerfile.roead
│   │   ├── Dockerfile.agent-runtime
│   │   └── docker-compose.yml
│   └── k8s/
│       ├── base/
│       └── overlays/
├── agents/                     # Pre-defined agent templates
│   ├── general-coder.md        # Stored in Fossil wiki
│   ├── bug-fixer.md
│   ├── reviewer.md
│   ├── docs-writer.md
│   └── test-writer.md
├── scripts/
│   └── ralph-wiggum.sh
├── go.mod
├── go.sum
└── Makefile
```

---

## Data Models

### Task (maps to Fossil Ticket)

```go
type Task struct {
    ID            string            `json:"id"`          // Fossil ticket UUID
    Title         string            `json:"title"`       // ticket.title
    Description   string            `json:"description"` // ticket.comment
    Status        TaskStatus        `json:"status"`      // ticket.status
    AgentType     string            `json:"agent_type"`  // ticket.type (repurposed)
    ExecutionMode ExecutionMode     `json:"execution_mode"`
    Model         string            `json:"model,omitempty"`
    RepoURL       string            `json:"repo_url,omitempty"`
    Branch        string            `json:"branch,omitempty"`
    Worktree      string            `json:"worktree,omitempty"`
    ParentID      *string           `json:"parent_id,omitempty"` // ticket.parent
    Priority      int               `json:"priority"`    // ticket.priority
    Labels        []string          `json:"labels"`      // ticket tags
    Secrets       *EncryptedPayload `json:"secrets,omitempty"` // age-encrypted
    CreatedAt     time.Time         `json:"created_at"`
    StartedAt     *time.Time        `json:"started_at,omitempty"`
    CompletedAt   *time.Time        `json:"completed_at,omitempty"`
}

type TaskStatus string
const (
    TaskPending   TaskStatus = "pending"   // Open
    TaskAssigned  TaskStatus = "assigned"  // Assigned
    TaskRunning   TaskStatus = "running"   // In Progress
    TaskCompleted TaskStatus = "completed" // Closed
    TaskFailed    TaskStatus = "failed"    // Closed (failed)
    TaskCancelled TaskStatus = "cancelled" // Closed (cancelled)
)
```

### Agent Definition (stored in Fossil Wiki)

```yaml
# Stored as wiki page: AgentDef_general-coder
id: general-coder
name: General Coder
description: General-purpose coding agent
base_runtime: claude-code
system_prompt: |
  You are a skilled software developer...
mcp_servers:
  - roea  # Always included
  - filesystem
  - github
default_model: claude-sonnet-4-20250514
resource_limits:
  max_turns: 50
  timeout_minutes: 30
  max_cost_usd: 5.00
```

### Encrypted Secrets

```go
type EncryptedPayload struct {
    Version    int    `json:"v"`          // Payload format version
    Recipient  string `json:"r"`          // age public key hint
    Ciphertext string `json:"c"`          // base64 age ciphertext
}

// Decrypted secrets structure
type TaskSecrets struct {
    APIKeys     map[string]string `json:"api_keys"`
    Credentials map[string]string `json:"credentials"`
    Tokens      map[string]string `json:"tokens"`
    Custom      map[string]string `json:"custom"`
}
```

---

## Fossil Integration Details

### Initialization

```bash
# Create new Roea instance
roead init --path ./my-project

# This runs:
fossil new ./my-project/roea.fossil
fossil open ./my-project/roea.fossil --workdir ./my-project/.roea

# Set up custom ticket fields
fossil ticket configure ...
```

### Custom Ticket Schema

```sql
-- Extended ticket fields for Roea
ALTER TABLE ticket ADD COLUMN agent_type TEXT;
ALTER TABLE ticket ADD COLUMN execution_mode TEXT;
ALTER TABLE ticket ADD COLUMN model TEXT;
ALTER TABLE ticket ADD COLUMN repo_url TEXT;
ALTER TABLE ticket ADD COLUMN branch TEXT;
ALTER TABLE ticket ADD COLUMN worktree TEXT;
ALTER TABLE ticket ADD COLUMN secrets_encrypted BLOB;
ALTER TABLE ticket ADD COLUMN started_at TEXT;
ALTER TABLE ticket ADD COLUMN completed_at TEXT;
```

### Direct SQLite Access

```go
func (s *FossilStore) queryTasks(filter TaskFilter) ([]Task, error) {
    db, err := sql.Open("sqlite3", s.fossilPath)
    if err != nil {
        return nil, err
    }
    defer db.Close()

    query := `
        SELECT
            tkt_uuid, title, status, type as agent_type,
            execution_mode, model, repo_url, branch,
            datetime(tkt_mtime, 'unixepoch') as updated_at
        FROM ticket
        WHERE status = ?
        ORDER BY tkt_mtime DESC
    `
    // ... execute and scan
}
```

### Artifact Storage

Task outputs and logs stored as Fossil attachments:

```go
func (s *FossilStore) StoreArtifact(taskID string, name string, data []byte) error {
    // fossil attachment add <filename> <ticket-uuid>
    cmd := exec.Command("fossil", "attachment", "add",
        "-t", taskID,
        tempFile.Name())
    return cmd.Run()
}
```

---

## Age Encryption Flow

### Server Setup

```go
func (s *Server) initializeCrypto() error {
    keyPath := filepath.Join(s.configDir, "roea.key")

    if !fileExists(keyPath) {
        // Generate new identity
        identity, err := age.GenerateX25519Identity()
        if err != nil {
            return err
        }

        // Save private key
        if err := os.WriteFile(keyPath, []byte(identity.String()), 0600); err != nil {
            return err
        }

        s.publicKey = identity.Recipient().String()
    }

    // Load existing identity
    data, _ := os.ReadFile(keyPath)
    s.identity, _ = age.ParseX25519Identity(string(data))
    s.publicKey = s.identity.Recipient().String()

    return nil
}
```

### Encrypting Secrets for Task

```go
func (s *Server) encryptSecretsForTask(secrets TaskSecrets) (*EncryptedPayload, error) {
    // For local execution: encrypt to self
    // For K8s/remote: encrypt to agent's ephemeral key

    plaintext, _ := json.Marshal(secrets)

    recipient, _ := age.ParseX25519Recipient(s.publicKey)

    var buf bytes.Buffer
    w, _ := age.Encrypt(&buf, recipient)
    w.Write(plaintext)
    w.Close()

    return &EncryptedPayload{
        Version:    1,
        Recipient:  s.publicKey[:8] + "...", // hint
        Ciphertext: base64.StdEncoding.EncodeToString(buf.Bytes()),
    }, nil
}
```

### Agent Receiving Secrets

```go
// In agent runtime, via MCP tool
func (a *Agent) getSecrets(taskID string) (*TaskSecrets, error) {
    // Call MCP tool
    resp := a.mcp.Call("roea_get_secrets", map[string]any{
        "task_id": taskID,
    })

    payload := resp.Secrets // EncryptedPayload

    // Decrypt with agent's identity (provided at spawn)
    ciphertext, _ := base64.StdEncoding.DecodeString(payload.Ciphertext)

    r, _ := age.Decrypt(bytes.NewReader(ciphertext), a.identity)
    plaintext, _ := io.ReadAll(r)

    var secrets TaskSecrets
    json.Unmarshal(plaintext, &secrets)

    return &secrets, nil
}
```

### K8s Secret Distribution

```yaml
# Agent pod spec
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: agent
    env:
    - name: ROEA_SERVER_PUBKEY
      value: "age1..." # Server's public key
    - name: ROEA_AGENT_IDENTITY
      valueFrom:
        secretKeyRef:
          name: roea-agent-keys
          key: identity
    volumeMounts:
    - name: age-identity
      mountPath: /etc/roea/identity
      readOnly: true
```

---

## MCP Server Tools

```yaml
tools:
  - name: roea_report_progress
    description: Report progress on current task
    parameters:
      message: string
      percent_complete: number (0-100)

  - name: roea_complete_task
    description: Mark current task as completed
    parameters:
      result_summary: string
      artifacts: array of file paths (stored in Fossil)

  - name: roea_fail_task
    description: Mark current task as failed
    parameters:
      error: string
      recoverable: boolean

  - name: roea_spawn_subtask
    description: Create a subtask (new Fossil ticket)
    parameters:
      title: string
      description: string
      agent_type: string (optional)

  - name: roea_get_secrets
    description: Get age-encrypted secrets for task
    parameters:
      task_id: string
    returns:
      encrypted_payload: EncryptedPayload

  - name: roea_store_artifact
    description: Store output file in Fossil
    parameters:
      task_id: string
      filename: string
      content: base64 string
```

---

## Ralph Wiggum Loop

```bash
#!/bin/bash
# ralph-wiggum.sh - Task selection loop

ROEA_API="${ROEA_API:-http://localhost:8080}"
AGENT_TYPE="${AGENT_TYPE:-general-coder}"

while true; do
    # Fetch next available task
    TASK=$(curl -s "$ROEA_API/api/v1/tasks/next?agent=$AGENT_TYPE")

    if [ "$TASK" = "null" ] || [ -z "$TASK" ]; then
        echo "No tasks available, sleeping..."
        sleep 10
        continue
    fi

    TASK_ID=$(echo "$TASK" | jq -r '.id')
    DESCRIPTION=$(echo "$TASK" | jq -r '.description')
    WORKTREE=$(echo "$TASK" | jq -r '.worktree')

    echo "Starting task: $TASK_ID"

    # Run the agent with MCP server configured
    cd "$WORKTREE"
    claude --mcp-server "roea:$ROEA_API/mcp?task=$TASK_ID" \
           --prompt "$DESCRIPTION" \
           --output-format json

    RESULT=$?

    if [ $RESULT -eq 0 ]; then
        echo "Task completed"
    else
        echo "Task failed: $RESULT"
    fi

    # Check for termination signal
    if [ -f "/tmp/roea-stop" ]; then
        echo "END"
        exit 0
    fi
done
```

---

## Implementation Phases

### Phase 1: Foundation
- [ ] Go project setup with module structure
- [ ] Fossil SCM integration (init, tickets, wiki, artifacts)
- [ ] Age encryption layer
- [ ] Core types and interfaces
- [ ] Basic REST API (task CRUD via Fossil tickets)
- [ ] Local executor (subprocess spawning)

### Phase 2: Agent Integration
- [ ] MCP server with basic tools
- [ ] Claude Code integration
- [ ] Agent definition YAML → Fossil wiki
- [ ] Git worktree management
- [ ] Task assignment logic

### Phase 3: UI & Real-time
- [ ] Next.js project setup
- [ ] Kanban board component (reads Fossil tickets)
- [ ] WebSocket for live updates
- [ ] Agent management UI
- [ ] Execution logs streaming

### Phase 4: K8s & Scaling
- [ ] K8s executor implementation
- [ ] Container images for agent runtimes
- [ ] Age key distribution to pods
- [ ] Fossil sync between instances
- [ ] Helm charts

### Phase 5: Advanced Features
- [ ] VM executor (QEMU)
- [ ] OpenCode and Codex integration
- [ ] Spec flow execution mode
- [ ] Model routing with cost tracking
- [ ] Subtask spawning

### Phase 6: Polish
- [ ] User authentication
- [ ] Audit logging (Fossil timeline)
- [ ] Backup/restore (Fossil clone)
- [ ] Documentation

---

## Configuration

```yaml
# roea.yaml
server:
  host: "0.0.0.0"
  port: 8080

fossil:
  path: "./roea.fossil"
  auto_sync: false  # Enable for distributed setup
  sync_url: ""      # Remote Fossil server

crypto:
  identity_path: "./roea.key"  # age identity
  # Public key derived automatically

executors:
  local:
    enabled: true
    max_concurrent: 4
    worktree_base: "/tmp/roea-worktrees"
  k8s:
    enabled: false
    namespace: "roea-agents"
    image: "ghcr.io/your-org/roea-agent-runtime:latest"
  vm:
    enabled: false
    qemu_path: "/usr/bin/qemu-system-x86_64"
    base_image: "./vm-images/agent-runtime.qcow2"

git:
  default_remote: "origin"
  branch_prefix: "roea/"
  auto_push: true

mcp:
  enabled: true

models:
  default: "claude-sonnet-4-20250514"
  providers:
    anthropic:
      api_key_encrypted: "age1..." # age-encrypted
    openai:
      api_key_encrypted: "age1..."
```

---

## Key Benefits of This Architecture

### Fossil SCM
- **Zero external dependencies** - No PostgreSQL, Redis, etc.
- **Portable** - Single `.fossil` file contains everything
- **Versioned** - Full audit trail of all changes
- **Syncable** - Can sync between local and remote instances
- **Battle-tested** - SQLite underneath, used by SQLite project itself

### Age Encryption
- **Simple** - No complex PKI infrastructure
- **Modern** - Designed by Filippo Valsorda (Go crypto team)
- **Composable** - Easy to encrypt to multiple recipients
- **Auditable** - Small, readable codebase

---

## Open Questions

1. **Fossil sync frequency** - How often to sync between distributed instances?
2. **Agent key rotation** - How often to rotate ephemeral agent keys?
3. **Conflict resolution** - When multiple agents modify same ticket?
4. **Cost tracking granularity** - Per-task or per-agent-invocation?
