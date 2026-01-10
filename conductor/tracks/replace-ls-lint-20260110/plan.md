# Plan: Replace ls-lint with Custom TypeScript Script

## Phase 1: Environment Setup & Tooling [checkpoint: 3a8f16f]
- [x] Task: Install required dependencies (`minimatch` and `@types/minimatch`).
- [x] Task: Create script skeleton at `scripts/lint-filenames.ts` and ensure it can be executed.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Environment Setup & Tooling' (Protocol in workflow.md)

## Phase 2: Configuration & File Discovery [checkpoint: dd05124]
- [x] Task: Implement `.ls-lint.yml` parsing using `js-yaml`.
- [x] Task: Implement `git ls-files` retrieval via `child_process`.
- [x] Task: Write unit tests for configuration parsing and file discovery.
- [x] Task: Conductor - User Manual Verification 'Phase 2: Configuration & File Discovery' (Protocol in workflow.md)

## Phase 3: Validation Logic Implementation [checkpoint: a7449a2]
- [x] Task: Implement ignore pattern matching using `minimatch`.
- [x] Task: Implement casing validation logic (kebab-case, snake_case, etc.) and rule mapping.
- [x] Task: Write unit tests for validation logic and casing rules.
- [x] Task: Conductor - User Manual Verification 'Phase 3: Validation Logic Implementation' (Protocol in workflow.md)

## Phase 4: Integration & Cleanup
- [ ] Task: Update `package.json` to use the new script in `check-filenames-root`.
- [ ] Task: Remove `@ls-lint/ls-lint` from `package.json` and devDependencies.
- [ ] Task: Run the new script against the entire codebase and verify it matches current `ls-lint` output.
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Integration & Cleanup' (Protocol in workflow.md)
