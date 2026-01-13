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

- [x] **Milestone 1.1**: Create Rust workspace structure (or efficient usage of
      `todo_mvp`).
- [x] **Milestone 1.2**: Port `TunnelState`, `Task`, and basic types to Rust
      structs.
  - **Destination**: `crates/tasklens-core`
  - **Source Types**:
    - `TunnelState` (`packages/tasklens/src/types/persistence.ts`)
    - `PersistedTask` (`packages/tasklens/src/persistence/schemas.ts`)
    - `Place` (`packages/tasklens/src/persistence/schemas.ts`)
    - `Schedule`, `RepeatConfig`, `TaskID`, `PlaceID`
  - **Strategy: Autosurgeon Compatibility via Serde**:
    - **Objective**: Use `autosurgeon` to bridge Rust structs and Automerge
      documents.
    - **Mechanism**: `autosurgeon` relies on `serde::Serialize` and
      `serde::Deserialize` traits to map Rust structs to Automerge document
      structures.
    - **Interim Step**: Implementing `serde` traits is the critical interim
      step. While we will verify these via JSON serialization tests, the
      ultimate goal is `transplant`ing these structs into an Automerge document.
    - **Validation**:
      - Serialize Rust structs to JSON.
      - Compare against expected JSON output from the TypeScript `TaskSchema`.
      - Use the "personal todos" dataset or a synthetic complex task tree as a
        test case to ensure exact structural alignment.
  - **Implementation Details**:
    - **[NEW] `crates/tasklens-core/src/types.rs`**:
      - Define `TaskID` and `PlaceID` as "NewType" constructs (tuple structs)
        for type safety.
      - Define enums: `TaskStatus`.
      - Define structs: `Schedule`, `RepeatConfig`, `PersistedTask`, `Place`,
        `TunnelState`.
      - derive `serde::Serialize` and `serde::Deserialize` for all types to
        match the existing JSON schema exactly.
    - **[MODIFY] `crates/tasklens-core/src/lib.rs`**:
      - Export `types` module.
    - Ensure strict compatibility with `automerge` via `autosurgeon` (or manual
      implementations where necessary).
  - **Verification**:
    - Create unit tests in `tasklens-core` that serialize these structs to JSON.
    - Validate that the generated JSON matches the structure expected by the
      TypeScript `TaskSchema`.
- [x] **Milestone 1.3**: Port `priority.ts` and `schedules` logic.
  - **Goal**: Implement purely functional domain logic in Rust.
  - **Source Files**: `priority.ts`, `routine-tasks.ts`, `dates.ts`,
    `visibility.ts`, `readiness.ts`, `feedback.ts`.
  - **Implementation Details**:
    - **[NEW] `crates/tasklens-core/src/utils/time.rs`**:
      - Implement `get_current_timestamp` and `get_interval_ms`.
      - _Note_: Domain logic will accept `current_time` as an argument.
    - **[NEW] `crates/tasklens-core/src/domain/`**:
      - `dates.rs`: Port `get_urgency_status` (returns `UrgencyStatus`),
        `is_same_day_utc`.
      - `readiness.rs`: Port `calculate_lead_time_factor`.
      - `visibility.rs`: Port `calculate_contextual_visibility` and
        `is_place_open`.
      - `feedback.rs`: Port `calculate_feedback_factors`.
      - `routine_tasks.rs`: Port
        `wake_up_routine_tasks(state: &mut TunnelState, current_time: u64)`.
      - `priority.rs`: Port `hydrate_task`, `recalculate_priorities`,
        `get_prioritized_tasks` (returns `Vec<ComputedTask>`).
    - **[MODIFY] `crates/tasklens-core/src/types.rs`**:
      - Add `UrgencyStatus` enum.
      - Add `EnrichedTask` (transient computation state), `Context`,
        `PriorityOptions`, `ViewFilter`, `ComputedTask`.
      - Add `ScheduleSource` enum (`Self`, `Ancestor`).
      - Add `OpenHours` struct/enum for location schedule parsing.
  - **Verification**:
    - [x] Unit tests for `UrgencyStatus` in `dates.rs`.
    - [x] Unit tests for `calculateLeadTimeFactor` in `readiness.rs`.
    - [x] Integration smoke test in `priority.rs`: Build a tree, prioritize,
          assert order.
- [x] **Milestone 1.4**: Implement a test runner to execute existing YAML BDD
      specs against the Rust domain crate.
  - _Success Criteria_: Rust domain tests pass the same scenarios as TS domain
    tests.
  - **Implementation Details**:
    - **[MODIFY] `crates/tasklens-core/Cargo.toml`**:
      - Add `serde_yaml_ng`, `anyhow`, `glob` to `dev-dependencies`.
    - **[NEW] `crates/tasklens-core/tests/compliance.rs`**:
      - **Structs**: Define `Feature`, `Background`, `Scenario`, `Step`,
        `Given`, `When`, `Then`, `TaskDef`, `TaskUpdate` matching the YAML
        schema.
      - **Harness**:
        - Iterate all files in
          `packages/tasklens/specs/compliance/fixtures/*.yaml`.
        - **Setup**: `TunnelState` from `background` + `given`.
        - **Execution**: Apply `when` mutations (status changes, etc.) and run
          `priority::recalculate_priorities`.
        - **Assertion**:
          - `expected_order`: Verify `TunnelState.view.task_order` matches IDs.
          - `expected_props`: Verify specific fields (e.g. `urgency_status`,
            `effective_due_date`) on `ComputedTask` or `PersistedTask`.
    - **[NOTE]**: Some YAMLs use `legacy_description` or implicit `when` blocks.
      The parser must be robust or we must clean up the YAMLs (prefer parser
      robustness/flexibility).
    - **Verification**:
      - [x] **Exhaustive Field Usage Check**: The test runner enforces that
            every field present in the YAML fixtures is both _defined_ in the
            Rust structs and _consumed_ by the test logic.
        - **Solution**:
          1.  Applied `#[serde(deny_unknown_fields)]` to all BDD structs.
          2.  Used **Exhaustive Destructuring** in the test runner logic to
              ensure all fields are matched.
      - [x] `cargo test` passes.
      - [x] `cargo fmt --check` passes.
      - [x] `pnpm check` passes.

### Epoch 2: The Walking Skeleton

_Goal: A minimal Dioxus app that permanently syncs data via Samod._

- [/] **Milestone 2.1**: `tasklens-store` Initialization & Samod Setup.
  - **Goal**: Create the persistence layer wrapping `samod`.
  - **Dependencies**: `samod`, `tasklens-core`, `autosurgeon`, `uuid` (v4),
    `serde`.
  - **Implementation Details**:
    - [x] **[MODIFY] `crates/tasklens-store/Cargo.toml`**:
      - Add `samod` (git dependency), `autosurgeon`, `automerge`.
    - [/] **[NEW] `crates/tasklens-store/src/store.rs`**:
      - [x] Define `AppStore` struct with `Repo` and `DocHandle`.
      - [x] Initialize `samod::Repo` in `AppStore::new()`.
      - [ ] Configure `IndexedDB` storage adapter (currently using
            `InMemoryStorage`).
        - **Concrete Steps**:
          1. Check `samod` for an IndexedDB storage adapter (e.g.,
             `samod::storage::IndexedDbStorage` or feature flag).
          2. If available, replace `InMemoryStorage::default()` with the
             IndexedDB adapter. Pass a database name (e.g., `"tasklens"`).
          3. If not available, check `automerge-repo` for a storage trait we can
             implement against `web-sys` IndexedDB bindings.
          4. Add `#[cfg(target_arch = "wasm32")]` conditional compilation to use
             IndexedDB in browser, `InMemoryStorage` in tests.
      - [x] **`get_state(&self) -> TunnelState`**: Hydrate state from the Doc
            using `autosurgeon::hydrate`. ✅ Implemented.
      - [ ] **`dispatch(&self, action: Action)`**: Apply changes to the Doc.
        - **Concrete Steps**:
          1. **[NEW] `crates/tasklens-store/src/actions.rs`**: Define the
             `Action` enum with variants for all mutations:
             ```rust
             pub enum Action {
                 CreateTask { parent_id: Option<TaskID>, title: String },
                 UpdateTask { id: TaskID, updates: TaskUpdates },
                 DeleteTask { id: TaskID },
                 CompleteTask { id: TaskID },
                 MoveTask { id: TaskID, new_parent_id: Option<TaskID> },
                 // ... additional actions as needed
             }
             ```
          2. **[MODIFY] `store.rs`**: Implement `dispatch` to:
             - Match on `Action` variant.
             - Call `handle.with_document()` to get mutable access.
             - Apply the mutation to `TunnelState` (clone, mutate, reconcile).
             - Commit the transaction.
          3. Add unit tests for each action variant.
- [ ] **Milestone 2.2**: Dioxus State Integration.
  - **Goal**: Inject the store into the Dioxus app and reflect state changes.
  - **Implementation Details**:
    - **[MODIFY] `crates/tasklens-ui/src/main.rs`**:
      - Initialize `AppStore` in `App` component (or passed in).
      - Use `use_context_provider` to share the store.
      - **Reactive State**:
        - Create a `Signal<TunnelState>` that mirrors the Automerge document.
        - Set up a subscription/listener to `AppStore` changes that updates this
          Signal.
- [ ] **Milestone 2.3**: Basic Task List Rendering.
  - **Goal**: Verify data loading by rendering a raw list.
  - **Implementation Details**:
    - **[NEW] `crates/tasklens-ui/src/components/debug_list.rs`**:
      - Simple `ul` / `li` loop iterating over `TunnelState.tasks`.
      - Display Task ID, Title, and Status.
- [ ] **Milestone 2.4**: Sync Verification.
  - **Goal**: Connect to the local sync server.
  - **Implementation Details**:
    - **[MODIFY] `crates/tasklens-store/src/store.rs`**:
      - Add WebSocket network adapter to `samod` config.
      - Connect to local sync server (e.g., `ws://localhost:8080`).
  - **Verification**:
    - Open React app, make changes (e.g., rename a task).
    - Refresh/Watch Dioxus app, see changes appear in `debug_list`.

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

1.  Implement **Milestone 2.1**: Initialize `tasklens-store` and integrate
    `samod`.
2.  Implement **Milestone 2.2**: Connect Dioxus to the store.
