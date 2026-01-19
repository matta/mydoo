# Specification: E2E Sync Framework & Verification

## Overview
Implement the End-to-End (E2E) testing infrastructure required to verify cross-browser synchronization for TaskLens. This involves creating a test harness that can spawn the `tasklens-sync-server`, manage multiple isolated browser contexts ("Alice" and "Bob"), and assert bidirectional data propagation.

## Functional Requirements

### 1. Sync Server Test Helper
- **Build Strategy:** The test harness must execute `cargo build -p tasklens-sync-server` before starting the tests to ensure the binary is up-to-date (Just-in-Time Build).
- **Process Management:**
  - Spawn the server process on a dynamic or configurable port (defaulting to a test-specific port, e.g., 3001) to avoid conflicts.
  - Ensure the process is cleanly terminated (killed) after tests complete, even on failure.

### 2. Multi-Context Test Spec (`sync.spec.ts`)
- **Structure:** Use a standard Playwright `.spec.ts` file utilizing `test.step` for narrative flow (Alice/Bob).
- **Scenario:**
  1.  **Setup:** Start Sync Server.
  2.  **Alice:** Open App -> Config Sync -> Status Green -> Create Task.
  3.  **Bob:** Open App -> Config Sync -> Status Green -> **Assert** Task appears.
  4.  **Bidirectional:** Bob completes Task -> **Assert** Alice sees completion.
- **Isolation:** Use `browser.newContext()` to ensure Alice and Bob have separate `LocalStorage` and `IndexedDB`.

## Non-Functional Requirements
- **Reliability:** Tests must be deterministic. Use Playwright's polling/waiting mechanisms (e.g., `expect.toPass`) rather than fixed sleeps when waiting for sync to propagate.
- **Performance:** Server build time should be minimized where possible (e.g., incremental builds).

## Acceptance Criteria
- [ ] `sync.spec.ts` passes consistently in CI and local environments.
- [ ] The `tasklens-sync-server` binary is automatically built and managed by the test runner.
- [ ] No zombie processes remain after test execution.

## Out of Scope
- Implementation of the `SyncIndicator` or `use_sync_client` (assumed to be present).
- Complex conflict resolution scenarios (MVP sync verification only).
