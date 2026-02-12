---
name: gh-pr-safe
description: Safely create or update GitHub pull requests from a shell without markdown escaping failures. Use body files and single-quoted heredocs instead of inline --body strings.
---

# Safe GitHub PR Creation

Use this skill when creating or editing a PR from the terminal with `gh`.

## Rules

1. Never pass markdown in `--body "..."`.
2. Always write PR body content to a file.
3. Always use a single-quoted heredoc delimiter (`<<'EOF'`) when creating that file.
4. Avoid shell-sensitive markdown in command arguments (`*`, backticks, `$()`, `[]`, `!`).
5. Verify the PR after creation/edit with `gh pr view`.

## Create PR Workflow

```bash
cat > /tmp/pr-body.md <<'EOF'
## Summary
- Item with `inline code`
- Item with wildcard text like test-e2e*

## Verification
- just verify
EOF

gh pr create \
  --base main \
  --head <branch> \
  --title "<title>" \
  --body-file /tmp/pr-body.md

gh pr view --json number,title,url,headRefName,baseRefName,state
```

## Edit Existing PR Workflow

```bash
cat > /tmp/pr-body-update.md <<'EOF'
## Update
- Added follow-up fix
EOF

gh pr edit <number-or-url> --body-file /tmp/pr-body-update.md

gh pr view <number-or-url> --json number,title,url,state
```

## Failure Recovery

If a create/edit command fails because content was interpreted by the shell:

1. Stop and inspect `git status` for unintended side effects.
2. Rewrite the body to a temp file using `<<'EOF'`.
3. Re-run only the `gh pr ... --body-file ...` command.
4. Re-check PR content with `gh pr view`.
