# Roea AI - Quick Start

## Prerequisites

- Go 1.21+
- Node.js 18+
- npm

## First Time Setup

```bash
# Build the backend
go build -o roead ./cmd/roead

# Initialize Roea (creates config, database, and keys)
./roead --init --path .

# Install frontend dependencies
cd web && npm install && cd ..
```

## Running the POC

### Terminal 1: Start Backend
```bash
./roead
```
Backend runs on http://localhost:8080

### Terminal 2: Start Frontend
```bash
cd web && npm run dev
```
Frontend runs on http://localhost:3000

## Using the UI

1. Open http://localhost:3000 in your browser
2. **Tasks tab**: View and create tasks on the Kanban board
3. **Agents tab**: View available agent definitions
4. **Monitor tab**: See system status and active executions

## API Examples

```bash
# Create a task
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "My Task", "description": "Task description", "agent_type": "general-coder"}'

# List all tasks
curl http://localhost:8080/api/v1/tasks

# List agents
curl http://localhost:8080/api/v1/agents

# Health check
curl http://localhost:8080/health
```

## Files Created

- `roea.yaml` - Configuration file
- `roea.fossil` - SQLite database for tasks
- `.roea/roea.key` - Age encryption key
- `.roea/worktrees/` - Git worktrees for agent execution
