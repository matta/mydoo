# E2E Tests for TaskLens UI

This directory contains Playwright-based end-to-end tests for the Dioxus
TaskLens UI application.

### Running Tests Locally

```bash
# Run all tests (desktop + mobile)
just test-e2e

# Run only desktop tests
just test-e2e-desktop

# Run with specific pattern (grep)
just test-e2e-desktop -- -g "Move Task"
```

### Manual Visual Verification (Bots + Humans)

Use the Dioxus dev server when you need to click through the app manually:

```bash
# Terminal 1 (from repo root)
just dev
```

Then open the URL printed by `dx serve` (typically `http://127.0.0.1:8080`) and
navigate to routes like `/plan`.

Do not use `python -m http.server` for this app; client-side routes (for
example `/plan`) will return 404 without SPA fallback routing.

For automated E2E runs, keep using `just test-e2e*`. Those commands build the
app and provide `WEB_DIST_DIR` so Playwright can run against its own local
server on `http://localhost:5180`.

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
└── tests-e2e-specs-task-moving-spec-ts-Task-Moving-Move-Task-to-Another-Parent-e2e-desktop/
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
├── specs/             # TypeScript Gherkin specs (.spec.ts)
├── fixtures.ts        # Test fixtures (I, debugFailure)
├── pages/             # Page Objects
├── steps/             # Step definitions (actor logic)
└── utils/             # Helpers (debug-utils.ts, etc.)
```

## Debug Utilities

The `debugFailure` fixture automatically captures DOM state when tests fail. See
`utils/debug-utils.ts` for the `dumpFailureContext` function which:

1. Serializes the DOM to a semantic JSON structure
2. Attaches it as `synthetic-dom.md` to the test report
