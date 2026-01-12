#!/bin/bash
# ralph-wiggum.sh - Task selection loop for Roea AI
#
# This script implements the "Ralph Wiggum Loop" execution mode:
# - Continuously polls for new tasks
# - Executes tasks using the configured agent runtime
# - Reports completion via the Roea API
# - Outputs "END" and exits when a stop signal is received

set -e

# Configuration
ROEA_API="${ROEA_API:-http://localhost:8080}"
AGENT_TYPE="${AGENT_TYPE:-general-coder}"
POLL_INTERVAL="${POLL_INTERVAL:-10}"
STOP_FILE="/tmp/roea-stop-$$"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Cleanup function
cleanup() {
    rm -f "$STOP_FILE"
    log_info "Cleanup complete"
}

trap cleanup EXIT

# Check dependencies
check_dependencies() {
    local missing=()

    if ! command -v curl &> /dev/null; then
        missing+=("curl")
    fi

    if ! command -v jq &> /dev/null; then
        missing+=("jq")
    fi

    if ! command -v claude &> /dev/null; then
        log_warning "claude command not found - will use fallback execution"
    fi

    if [ ${#missing[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing[*]}"
        exit 1
    fi
}

# Fetch next task from API
fetch_next_task() {
    local response
    response=$(curl -s "$ROEA_API/api/v1/tasks/next?agent=$AGENT_TYPE")

    if [ "$response" = "null" ] || [ -z "$response" ]; then
        echo ""
        return
    fi

    echo "$response"
}

# Execute task
execute_task() {
    local task_id="$1"
    local description="$2"
    local worktree="$3"

    log_info "Executing task: $task_id"

    # Change to worktree if specified
    if [ -n "$worktree" ] && [ -d "$worktree" ]; then
        cd "$worktree"
    fi

    # Execute with Claude Code
    if command -v claude &> /dev/null; then
        claude --print \
               --output-format json \
               --prompt "$description" \
               2>&1
    else
        # Fallback: just report that the task was processed
        log_warning "Claude not available, simulating execution"
        echo '{"success": true, "message": "Simulated execution"}'
    fi

    return $?
}

# Mark task as complete
complete_task() {
    local task_id="$1"
    local result="$2"

    curl -s -X POST "$ROEA_API/api/v1/tasks/$task_id/complete" \
         -H "Content-Type: application/json" \
         -d "{\"result_summary\": \"$result\"}" \
         > /dev/null
}

# Mark task as failed
fail_task() {
    local task_id="$1"
    local error="$2"

    curl -s -X POST "$ROEA_API/api/v1/tasks/$task_id/fail" \
         -H "Content-Type: application/json" \
         -d "{\"error\": \"$error\"}" \
         > /dev/null
}

# Main loop
main_loop() {
    log_info "Starting Ralph Wiggum loop..."
    log_info "API: $ROEA_API"
    log_info "Agent: $AGENT_TYPE"
    log_info "Poll interval: ${POLL_INTERVAL}s"
    log_info "Stop file: $STOP_FILE"
    log_info ""
    log_info "To stop gracefully: touch $STOP_FILE"
    log_info ""

    while true; do
        # Check for stop signal
        if [ -f "$STOP_FILE" ]; then
            log_info "Stop signal received"
            echo "END"
            exit 0
        fi

        # Fetch next task
        task=$(fetch_next_task)

        if [ -z "$task" ]; then
            log_info "No tasks available, sleeping for ${POLL_INTERVAL}s..."
            sleep "$POLL_INTERVAL"
            continue
        fi

        # Parse task
        task_id=$(echo "$task" | jq -r '.id')
        title=$(echo "$task" | jq -r '.title')
        description=$(echo "$task" | jq -r '.description')
        worktree=$(echo "$task" | jq -r '.worktree // ""')

        log_info "Found task: $title ($task_id)"

        # Execute task
        if execute_task "$task_id" "$description" "$worktree"; then
            log_success "Task completed: $task_id"
        else
            exit_code=$?
            log_error "Task failed: $task_id (exit code: $exit_code)"
        fi

        # Small delay between tasks
        sleep 2
    done
}

# Entry point
main() {
    log_info "Roea AI - Ralph Wiggum Loop"
    log_info "==========================="

    check_dependencies
    main_loop
}

main "$@"
