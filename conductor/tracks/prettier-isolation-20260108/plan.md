# Plan: Enforce Strict Prettier Isolation

## Phase 1: Root Isolation

- [x] Task: Update root `.prettierignore` to exclude all subdirectories [32f84ab]
- [ ] Task: Conductor - User Manual Verification 'Root Isolation' (Protocol in
      workflow.md)

## Phase 2: Sub-package Opt-In

- [ ] Task: Configure Prettier for `apps/client` (add/update `.prettierignore`)
- [ ] Task: Configure Prettier for `packages/tasklens` (add/update
      `.prettierignore`)
- [ ] Task: Conductor - User Manual Verification 'Sub-package Opt-In' (Protocol
      in workflow.md)

## Phase 3: Final Verification

- [ ] Task: Verify that root Prettier command touches no files in subdirectories
- [ ] Task: Verify that sub-package Prettier commands still function correctly
- [ ] Task: Conductor - User Manual Verification 'Final Verification' (Protocol
      in workflow.md)
