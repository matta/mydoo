# Implementation Plan: E2E Sync Framework & Verification

## Phase 1: Sync Server Test Harness [checkpoint: 29c9d1d]
Implement the utility to manage the lifecycle of the `tasklens-sync-server` during E2E tests.

- [x] Task: Create `SyncServerHelper` utility ccdd01e
    - [x] Implement `build_server()` using `child_process.exec` (JIT Build).
    - [x] Implement `start_server()` with configurable port.
    - [x] Implement `stop_server()` to ensure clean termination.
- [x] Task: Integrate Sync Server with Playwright e5766eb
    - [x] Add global `beforeAll` / `afterAll` hooks to manage the server process across the test suite.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Sync Server Test Harness' (Protocol in workflow.md)

## Phase 2: Multi-Context Infrastructure [~]
Extend the Playwright environment to support multiple isolated users (Alice and Bob).

- [~] Task: Define Multi-User Fixtures
    - [ ] Create a fixture that provides two independent browser contexts.
    - [ ] Implement helper methods to switch focus/actions between Alice and Bob.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Multi-Context Infrastructure' (Protocol in workflow.md)

## Phase 3: Sync Verification Spec
Implement and verify the actual synchronization flow.

- [ ] Task: Implement `sync.spec.ts`
    - [ ] Implement "Alice creates task" flow.
    - [ ] Implement "Bob observes task" flow.
    - [ ] Implement "Bob completes task" flow.
    - [ ] Implement "Alice observes completion" flow.
- [ ] Task: Verify determinism
    - [ ] Ensure tests use `expect.toPass` or similar polling for sync propagation.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Sync Verification Spec' (Protocol in workflow.md)
