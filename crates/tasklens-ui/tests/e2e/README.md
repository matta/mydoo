# E2E Tests for TaskLens UI

This directory contains Playwright-based end-to-end tests for the Dioxus
TaskLens UI application.

## Running Tests

```bash
# From project root
pnpm --filter @mydoo/tasklens-ui test-e2e

# With specific project
pnpm --filter @mydoo/tasklens-ui test-e2e -- --project=bdd-desktop

# With pattern matching
pnpm --filter @mydoo/tasklens-ui test-e2e -- -g "Move Task"
```

## Test Output Locations

Playwright saves test outputs to these directories (relative to
`crates/tasklens-ui/`):

| Directory            | Purpose                                              |
| -------------------- | ---------------------------------------------------- |
| `test-results/`      | Per-test output directories with failure attachments |
| `playwright-report/` | HTML report for browsing test results                |

### Test Results Structure

Each failed test creates a subdirectory in `test-results/` with:

- `synthetic-dom.md` — DOM snapshot at failure time (semantic structure
  optimized for debugging)
- `error-context.md` — Additional failure context
- Screenshots/traces (when configured)

Example path:

```
test-results/
└── tests-e2e-features-task-mo-f6890-Move-Task-to-Another-Parent-bdd-desktop/
    ├── synthetic-dom.md
    └── error-context.md
```

### Viewing the HTML Report

```bash
cd crates/tasklens-ui
pnpm exec playwright show-report
```

Or open `crates/tasklens-ui/playwright-report/index.html` directly.

> **Note:** The `test-results/` directory is cleaned at the start of each test
> run. If tests pass, the directory will be empty or contain only
> `.last-run.json`.

## Directory Structure

```
tests/e2e/
├── .features-gen/     # Generated BDD tests (gitignored)
├── features/          # Gherkin .feature files
├── fixtures.ts        # Test fixtures (plan, debugFailure)
├── pages/             # Page Objects
├── steps/             # Step definitions
└── utils/             # Helpers (debug-utils.ts, etc.)
```

## Debug Utilities

The `debugFailure` fixture automatically captures DOM state when tests fail. See
`utils/debug-utils.ts` for the `dumpFailureContext` function which:

1. Serializes the DOM to a semantic JSON structure
2. Attaches it as `synthetic-dom.md` to the test report
