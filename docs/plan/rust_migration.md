# Migration Plan: React to Rust + Dioxus

## Executive Summary

This document outlines the strategy for migrating the current React/TypeScript
application (`apps/client`) to a Rust/Dioxus implementation. The goal is to
achieve feature parity while maintaining the local-first architecture usage
`automerge-repo` (specifically the `samod` Rust implementation).

The migration will be performed in phases (Epochs) to ensure stability and allow
for incremental verification. Both implementations will coexist until the Rust
version is fully feature-complete and verified.

## Architecture Vision

### Current (React)

- **Domain:** TypeScript (`packages/tasklens/src/domain`) - Pure functions for
  priority, visibility, etc.
- **State:** Redux + Automerge Repo (JS).
- **UI:** React + Mantine.
- **Persistence:** IndexedDB (via `automerge-repo`).

### Target (Rust)

- **Domain:** Rust Crate (`tasklens-rs` or similar) - Port of domain logic.
- **State:** Dioxus State Management (Signals/Context) + `samod` (Automerge Repo
  in Rust).
- **UI:** Dioxus (WASM).
- **Persistence:** IndexedDB (via `samod`/`automerge-repo` storage adapters).

## Subsystems & Isolation

We can identify three major layers that can be tackled somewhat independently:

1.  **Core Domain Logic (`tasklens-core`)**:
    - **Description**: The "brain" of the app. Priority sorting, recurrent task
      logic, visibility filters.
    - **Porting Strategy**: Direct port of `packages/tasklens/src/domain` types
      and functions to Rust.
    - **Verification**: Unit tests matching the logic of existing TS tests. Can
      reuse BDD YAML fixtures.

2.  **Persistence & Sync (`tasklens-store`)**:
    - **Description**: Handling Automerge documents, loading/saving to
      IndexedDB, syncing over WebSocket.
    - **Porting Strategy**: Integrate `samod`. Replace Redux middleware with a
      Dioxus-friendly store/controller that wraps the `samod` Repo.

3.  **User Interface (`tasklens-ui`)**:
    - **Description**: The views and components.
    - **Porting Strategy**: Port React components to Dioxus rsx. Since Dioxus is
      React-like, this is a structural translation. CSS/Tailwind can be largely
      reused.

## Directory Structure

To support the migration and coexistence, we will utilize a Cargo Workspace at
the root of the repository. All new Rust code will live in `crates/` to keep the
migration clean and isolated.

```text
/
├── Cargo.toml              # Workspace Root
├── apps/
│   └── client/             # Existing React implementation
├── crates/                 # [NEW] Rust Workspace Members
│   ├── tasklens-core/      # Domain logic (pure Rust)
│   ├── tasklens-store/     # Persistence & Sync logic
│   └── tasklens-ui/        # [NEW] Dioxus WASM implementation
├── packages/               # Existing Node.js packages
└── ...
```

### Cargo Workspace Setup

The root `Cargo.toml` will define the workspace members:

```toml
[workspace]
resolver = "2"
members = [
  "crates/*",
]

[workspace.dependencies]
# Shared dependencies like serde, chrono, uuid, etc.
```

## Epochs & Milestones

### Epoch 1: Foundation & Domain Parity

_Goal: Working Rust domain logic and basic persistence._

- [ ] **Milestone 1.1**: Create Rust workspace structure (or efficient usage of
      `todo_mvp`).
- [ ] **Milestone 1.2**: Port `TunnelState`, `Task`, and basic types to Rust
      structs.
- [ ] **Milestone 1.3**: Port `priority.ts` and `schedules` logic.
- [ ] **Milestone 1.4**: Implement a test runner to execute existing YAML BDD
      specs against the Rust domain crate.
  - _Success Criteria_: Rust domain tests pass the same scenarios as TS domain
    tests.

### Epoch 2: The Walking Skeleton

_Goal: A minimal Dioxus app that syncs data._

- [ ] **Milestone 2.1**: Integrate `samod` into the Dioxus app.
- [ ] **Milestone 2.2**: Implement basic "Load" and "Save" to IndexedDB.
- [ ] **Milestone 2.3**: Render a raw list of tasks (no fancy UI) from the
      persisted state.
- [ ] **Milestone 2.4**: Verify sync with the existing Sync Server.

### Epoch 3: Feature Parity (The Grind)

_Goal: Porting UI components to match functionality._

- [ ] **Milestone 3.1**: Task List & Inspection (Read-only view matches React).
- [ ] **Milestone 3.2**: Task Creation & Editing (Forms, Mutators).
- [ ] **Milestone 3.3**: Drag & Drop / Reordering (if applicable).
- [ ] **Milestone 3.4**: Settings & Configuration.

### Epoch 4: Verification & Switchover

_Goal: rigorous testing and final cutover._

- [ ] **Milestone 4.1**: Point Playwright E2E tests at the Rust WASM app.
  - _Strategy_: Run Dioxus app on a local port (e.g., 8080) and update
    `playwright.config.ts` to test against it.
- [ ] **Milestone 4.2**: Fix UI discrepancies (CSS, accessibility labels) to
      make tests pass.
- [ ] **Milestone 4.3**: Performance tuning.
- [ ] **Milestone 4.4**: "Retire" React app (remove from build, archive code).

## Testing Strategy

We will leverage the high investment in existing tests:

1.  **YAML BDD Specs (`specs/compliance/*.yaml`)**:
    - These describe domain rules independent of language.
    - _Action_: Write a Rust harness to parse these YAMLs and run the assertions
      against `tasklens-core`.

2.  **Playwright E2E**:
    - These test the DOM.
    - _Action_: Ensure Dioxus renders semantic HTML with compatible classes/IDs
      (e.g., `data-testid`).
    - _Benefit_: If the DOM matches, we get "free" full-stack verification.

3.  **Unit Tests**:
    - Write Rust unit tests (`#[test]`) for any new Rust-specific glue code.

## Risks & Mitigations

- **`samod` Maturity**: As an experimental repo, `samod` might lack features or
  stability.
  - _Mitigation_: Early POC in Epoch 2. Fallback to `automerge-repo-rs` if
    `samod` is blocked, though `samod` is explicitly requested.
- **WASM Bridge Overhead**: Heavy DOM manipulation might be slower or different
  in Dioxus.
  - _Mitigation_: Dioxus is generally fast. Avoid excessive JS interop. Use
    `web-sys` carefully.
- **Ecosystem Gaps**: React libraries (e.g., Mantine) don't exist in Dioxus.
  - _Mitigation_: We are using Tailwind, so styling is portable. Complex
    components (DatePickers, Comboboxes) will need to be built or found in the
    Dioxus ecosystem (headless UI libraries).

## Next Steps

1.  Initialize the `tasklens-core` Rust crate.
2.  Begin porting `TunnelState` types.
3.  Set up the YAML test harness in Rust.
