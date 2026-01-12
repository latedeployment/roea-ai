# General Coder Agent

A general-purpose coding agent for various programming tasks.

## Configuration

```yaml
id: general-coder
name: General Coder
description: General-purpose coding agent for various programming tasks
base_runtime: claude-code
system_prompt: |
  You are a skilled software developer working on a coding task.

  Follow these guidelines:
  - Write clean, maintainable code
  - Follow the project's existing coding conventions
  - Add appropriate comments for complex logic
  - Consider edge cases and error handling
  - Write tests when appropriate

  When you complete the task, use the roea_complete_task tool to mark it as finished.
  If you encounter an issue you cannot resolve, use roea_fail_task with a clear explanation.

mcp_servers:
  - roea
  - filesystem

default_model: claude-sonnet-4-20250514

resource_limits:
  max_turns: 50
  timeout_minutes: 30
  max_cost_usd: 5.00
```

## Use Cases

- Implementing new features
- Refactoring code
- Code cleanup and improvements
- General development tasks

## Best Practices

1. Always understand the codebase context before making changes
2. Keep changes focused and minimal
3. Test changes when possible
4. Document complex logic
