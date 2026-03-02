---
name: Pebble Task Tracker
description: Instructions for interacting with the `pebble` task tracking tool.
---

# Pebble Skill

This skill provides guidelines and common commands for interacting with the **Pebble** task tracker.

## 🚨 CRITICAL RULES

1.  **NEVER MANUALLY EDIT YAML FRONTMATTER IN TASK FILES**.
    - Pebble stores tasks as markdown files in `docs/pebble/`. You must NOT edit them directly.
    - ALWAYS use the `pebble` CLI tool to read, create, or modify tasks.
    - The `pebble` tool ensures graph integrity and proper metadata handling.

2.  **FREELY EDIT THE BODY OF TASK FILES**.
    - The body of the task file is plain markdown and can be edited directly.
    - This is where you should write your task descriptions, notes, and other information.
    - Use checkmarks for subtasks: `- [ ] Subtask 1`

3.  **PREFER JSON OUTPUT**:
    - ALWAYS include the `--json` flag when running `pebble` commands.
    - This ensures reliable, machine-readable data and avoids parsing issues from standard output.

4.  **Verify Operations**:
    - After creating or updating tasks, verify the result using `pebble show <ID> --json`.

## Common Commands

### Creating Tasks

Use `pebble add` to create new tasks.

```bash
# Create a basic task
pebble add "Task Title" --json

# Create with priority and tags
pebble add "Task Title" --priority 10 --tag bug --tag high-priority --json

# Create with prerequisites (dependencies)
pebble add "Child Task" --need <PARENT_ID> --json

# Create as a prerequisite for another task
pebble add "Pre-requisite Task" --blocks <DEPENDENT_ID> --json
```

**Important Flags:**

- `--status`: `todo`, `in_progress`, `done`, `canceled`. (Default: `todo`)
- `--priority`: `0` (Highest) to `99` (Lowest).
- `--need`: ID of a prerequisite task (can be repeated).
- `--tag`: Add a tag (can be repeated).

### Viewing Tasks

```bash
# List all active tasks
pebble list --json

# List with filters
pebble list --status todo --tag backend --json

# Show detailed view of a specific task
pebble show <ID> --json

# Search for tasks by text
pebble search "search term" --json
```

### Updating Tasks

```bash
# Update title, priority or status
pebble update <ID> --title "New Title" --status in_progress --json

# Add/Remove tags and dependencies
pebble update <ID> --add-tag new-tag --remove-tag old-tag --add-need <OTHER_ID> --json

# Append content to the body
pebble update <ID> --append-body "This is an additional note." --json
```

### Terminal Operations

```bash
# Mark a task as done
pebble update <ID> --status done --json

# Archive old closed tasks
pebble archive --json
```

## Troubleshooting

- If a command fails, check `pebble --help`.
- Use `pebble check` to verify the health of the task graph.
