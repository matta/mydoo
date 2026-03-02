---
id: issue-sev6lf175mk
title: >-
  Evaluate cargo update workflow from cargo-semver-checks for Rust dependency
  updates
status: todo
priority: 40
created_at: 2026-03-02T14:59:35.596724301+00:00
tags:
  - task
---
Investigate using a GitHub Actions workflow similar to https://github.com/obi1kenobi/cargo-semver-checks/blob/main/.github/workflows/dependencies.yml to update Rust packages via 'cargo update' instead of, or in addition to, renovate.

Key points to evaluate:

- The cargo-semver-checks approach runs 'cargo update' on a schedule
- This may provide better compatibility than renovate's individual package bumps
- Could reduce PR noise by batching Cargo.lock updates
- Consider semver compatibility checking as part of the workflow

## Related Issues

- mydoo-olt
- mydoo-paa
