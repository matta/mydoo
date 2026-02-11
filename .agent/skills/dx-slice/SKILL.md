---
name: dx-slice
description: Use when choosing the next chunk or vertical slice for the Dioxus Components migration. Read docs/design/dioxus-components-migration.todo.md and docs/design/dioxus-components-migration.md, then propose the next slice from those docs.
---

# Dioxus Migration Slice Planner

When asked to choose the next migration chunk/slice, do this:

1. Read `docs/design/dioxus-components-migration.todo.md`.
2. Read `docs/design/dioxus-components-migration.md`.
3. Recommend the next slice based on those docs.
4. Keep recommendations aligned with the active chunk, class debt register, and migration inventory in those docs.

## Git Branch Rules

1. Always fetch latest refs before starting slice work: `git fetch origin`.
2. Always merge `origin/main` into the current working branch before starting: `git merge origin/main`.
3. If currently on a feature branch (not `main`), create the next branch from that feature branch to support stacked PRs.
4. Use `codex/` branch names for new slice branches.
