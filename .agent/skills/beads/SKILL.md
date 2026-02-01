---
name: Beads Issue Tracker
description: Instructions for interacting with the `beads` (bd) issue tracking tool.
---

# Beads (bd) Skill

This skill provides guidelines and common commands for interacting with the **Beads** issue tracker using the `bd` command-line tool.

## 🚨 CRITICAL RULES

1.  **NEVER MANUALLY EDIT JSONL FILES**.
    -   The `.beads/*.jsonl` files are the storage layer, but you must NOT edit them directly.
    -   ALWAYS use the `bd` CLI tool to read, create, or modify issues.
    -   The `bd` tool is the single source of truth.

2.  **Verify Creation**:
    -   After creating issues, always verify they exist using `bd show <ID>` or `bd list`.

## Common Commands

### Creating Issues

Use `bd create` (or `bd new`) to make new beads.

```bash
# Create a basic task
bd create --type task --title "Task Title" --description "Detailed description." --priority 2

# Create an EPIC
bd create --type epic --title "Epic Title" --description "High level goal." --priority 2

# Create a child task (sub-task)
bd create --type task --title "Child Task" --parent <PARENT_ID> --priority 2

# Create silently (good for scripts/batch, outputs only ID)
bd create --type task --title "Refactoring" --silent
```

**Important Flags:**
-   `--type`: `task`, `bug`, `epic`, `chore`, etc. (Default: `task`)
-   `--priority`: `0` (Critical) to `4` (Low). (Default: `2`)
-   `--parent`: ID of the parent issue (e.g., `mydoo-123`).
-   `--dependencies`: explicit dependencies (e.g. `blocks:mydoo-123`).

### Viewing Issues

```bash
# List all open issues
bd list

# Show detailed view of a specific issue
bd show <ISSUE_ID>

# List children of a specific issue
bd children <ISSUE_ID>
```

### Updating Issues

```bash
# Close an issue
bd close <ISSUE_ID>

# Update title or status
bd update <ISSUE_ID> --title "New Title"
```

## Troubleshooting

-   If a command fails, check `bd --help`.
-   If you cannot find an issue you just thought you created, check for exit codes of `1` in previous commands.
