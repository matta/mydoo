# Spec: Enforce Strict Prettier Isolation

## Goal

Configure Prettier so that the root command _only_ formats root files, and all
sub-projects must explicitly opt-in to formatting. This prevents root-level
Prettier commands from impacting the entire monorepo and enforces an "opt-in"
policy for sub-packages.

## Requirements

1.  **Root Isolation:** The root `.prettierignore` must be configured to exclude
    all subdirectories (e.g., using `*/*` or a similar pattern).
2.  **Opt-In for Sub-packages:** Every directory containing a `package.json`
    that defines Prettier scripts must be configured to handle its own
    formatting.
3.  **Local Configuration:** Sub-packages must have their own `.prettierignore`
    files to whitelist their own contents and manage their own exclusions.
4.  **Verification:** Running `prettier --write .` at the root must not touch
    any files in any subdirectory.

## Implementation Details

- Modify `.prettierignore` in the root directory.
- Update/Create `.prettierignore` in `apps/client`.
- Update/Create `.prettierignore` in `packages/tasklens`.
- Ensure that running Prettier in `apps/client` and `packages/tasklens` still
  works correctly.

## Verification Plan

1.  Run `pnpm fix-format-root` (or the equivalent command from the root
    `package.json`) and verify that no files in subdirectories are modified.
2.  Run `pnpm --filter @mydoo/client fix-format` and verify it formats files
    within `apps/client`.
3.  Run `pnpm --filter @mydoo/tasklens fix-format` and verify it formats files
    within `packages/tasklens`.
