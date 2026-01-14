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

- **Domain:** Rust Crate (`tasklens-core`) - Port of domain logic.
- **State:** Dioxus State Management + `tasklens-store` (Custom Automerge
  Wrapper).
- **UI:** Dioxus (WASM).
- **Persistence:** IndexedDB (via `automerge` storage adapters).
- **Sync:** Custom WebSocket Server (`sync-server`) + Client `SyncService`.

## Subsystems & Isolation

We can identify three major layers that can be tackled somewhat independently:

1.  **Core Domain Logic (`tasklens-core`)**:
    - **Description**: The "brain" of the app. Priority sorting, recurrent task
      logic, visibility filters.
    - **Porting Strategy**: Direct port of `packages/tasklens/src/domain` types
      and functions to Rust.
    - **Verification**: Unit tests matching the logic of existing TS tests. Can
      reuse BDD YAML fixtures.

2.  **Persistence & Sync (`tasklens-store`, `sync-protocol`, `sync-server`)**:
    - **Description**: Handling Automerge documents, loading/saving to
      IndexedDB, syncing over WebSocket.
    - **Pivot Decision**: We are moving away from `samod` to a custom
      implementation (ported from `todo_mvp`) to avoid WASM incompatibility
      issues.
    - **Porting Strategy**:
      - `sync-protocol`: Shared types for encryption and messaging.
      - `sync-server`: Axum-based WebSocket server.
      - `tasklens-store`: Manages the `automerge` document and connects to the
        sync server.

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
│   ├── tasklens-store/     # Persistence & Networking
│   ├── tasklens-ui/        # Dioxus WASM implementation
│   ├── sync-protocol/      # [NEW] Shared sync types
│   └── sync-server/        # [NEW] WebSocket server
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

_Goal: A minimal Dioxus app that permanently syncs data via our custom Sync
Server._

- [x] **Milestone 2.1**: `tasklens-store` Initialization (Local).
  - **Goal**: Create the local persistence layer wrapping `automerge`.
  - **Status**: Completed (re-verified against new plan). The current `Store`
    implementation in `tasklens-store` matches the requirements (direct
    `automerge` usage).
  - **Note**: The original plan used `samod`. We have effectively implemented
    the "custom wrapper" part already. We just need to ensure it aligns with
    `todo_mvp`'s `store.rs`.

- [x] **Milestone 2.2**: Sync Protocol (`sync-protocol`).
  - **Goal**: Define shared types and protocol for syncing.
  - **Source**: `todo_mvp/sync_protocol`
  - **Implementation Details**:
    - [x] Create directory: `crates/sync-protocol`
    - [x] Create `crates/sync-protocol/Cargo.toml`:
      - [x] Add `[package]` section (name="sync_protocol", version="0.1.0",
            edition="2021").
      - [x] Add `[dependencies]` section with
            `serde = { version = "1.0", features = ["derive"] }`.
    - [x] Create `crates/sync-protocol/src/lib.rs`:
      - [x] Add `use serde::{Deserialize, Serialize};`.
      - [x] Implement `EncryptedBlob` struct:
        - [x] Field: `nonce: [u8; 24]` (XChaCha20 uses 24-byte nonce).
        - [x] Field: `ciphertext: Vec<u8>`.
        - [x] Derive: `Debug, Clone, Serialize, Deserialize`.
      - [x] Implement `ClientMessage` enum:
        - [x] Variant
              `Hello { client_id: String, sync_id: String, last_sequence: i64 }`.
        - [x] Variant
              `SubmitChange { sync_id: String, payload: EncryptedBlob }`.
        - [x] Derive: `Debug, Clone, Serialize, Deserialize`.
      - [x] Implement `ServerMessage` enum:
        - [x] Variant
              `ChangeOccurred { sequence_id: i64, sync_id: String, source_client_id: String, payload: EncryptedBlob }`.
        - [x] Derive: `Debug, Clone, Serialize, Deserialize`.

- [x] **Milestone 2.3**: Sync Server (`sync-server`).
  - **Goal**: Implement the WebSocket sync server.
  - **Source**: `todo_mvp/sync_server`
  - **Implementation Details**:
    - **Step 1: Crate Setup**
      - [x] Create directory: `crates/sync_server`
      - [x] Create `crates/sync_server/Cargo.toml` (deps: axum, tokio, rusqlite,
            etc).
      - [x] Create `crates/sync_server/src/main.rs` (skeleton).
    - **Step 2: Database**
      - [x] Create `crates/sync_server/src/db.rs`.
      - [x] Implement `init_pool`, `append_update`, `get_changes_since`.
    - **Step 3: Server Logic**
      - [x] Implement `AppState` and `main` (setup router).
      - [x] Implement `ws_handler` and `handle_socket` (handshake + loop).
  - **Verification**:
    - [x] Run `cargo run -p sync_server`.

- [ ] **Milestone 2.4**: Store Refactor & Samod Removal (`tasklens-store`).
  - **Goal**: Remove `samod` and use `automerge` directly.
  - **Implementation Details**:
    - [ ] **[MODIFY] `crates/tasklens-store/Cargo.toml`**:
      - Remove `samod` dependency.
      - Ensure `automerge`, `autosurgeon`, `unfold` are available.
    - [ ] **[Refactor] `crates/tasklens-store/src/store.rs`**:
      - Replace `Repo` usage with `automerge::AutoCommit`.
      - Remove `start_sync`.
      - Implement `subscribe` using internal signals or polling.
    - [ ] **[Cleanup] Workspace**:
      - Remove `samod` from root `Cargo.toml`.
  - **Verification**:
    - [ ] `cargo check -p tasklens-store`.

- [ ] **Milestone 2.5**: Client Networking (`tasklens-store`).
  - **Goal**: Connect the client `Store` to the `sync-server`.
  - **Source**: `todo_mvp`
  - **Implementation Details**:
    - **[NEW] `crates/tasklens-store/src/crypto.rs`**:
      - Port `todo_mvp/src/crypto.rs` (Encryption/Decryption logic using
        `chacha20poly1305`, Key Derivation).
    - **[NEW] `crates/tasklens-store/src/network.rs`**:
      - Port `todo_mvp/src/network.rs` (`SyncService`, WebSocket loop, retry
        logic).
      - Dependence: `gloo-net`, `web-sys`, `wasm-bindgen-futures`.
    - **[MODIFY] `crates/tasklens-store/src/store.rs`**:
      - Expose methods needed for sync (extract changes, apply changes).
      - Integrate `run_sync_loop` or expose it for the UI to consume.
  - **Verification**:
    - Add unit tests for `crypto.rs`.
    - Integration test: Ensure `SyncService` can talk to a running
      `sync-server`.

- [ ] **Milestone 2.6**: Dioxus Integration (UI Connection).
  - **Goal**: Hook up the sync loop in the Dioxus app.
  - **Implementation Details**:
    - **[MODIFY] `crates/tasklens-ui/src/main.rs`**:
      - Initialize `SyncService` / start sync loop in a coroutine.
      - Pass the Master Key (hardcoded or from local storage for now).
    - **[NEW] `crates/tasklens-ui/components/debug_list.rs`**:
      - Basic list to verify data sync.

### Epoch 3: Feature Parity (The Grind)

_Goal: Porting UI components to match functionality._

- [ ] **Milestone 3.1**: Task List & Inspection (Read-only view matches React).
- [ ] **Milestone 3.2**: Task Creation & Editing (Forms, Mutators).
- [ ] **Milestone 3.3**: Drag & Drop / Reordering.
- [ ] **Milestone 3.4**: Settings & Configuration.

### Epoch 4: Verification & Switchover

_Goal: rigorous testing and final cutover._

- [ ] **Milestone 4.1**: Playwright E2E tests.
- [ ] **Milestone 4.2**: Fix UI discrepancies.
- [ ] **Milestone 4.3**: Performance tuning.
- [ ] **Milestone 4.4**: "Retire" React app.

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
  - _Mitigation_: We are using Tailwind, so styling is portable. Complex
    components (DatePickers, Comboboxes) will need to be built or found in the
    Dioxus ecosystem (headless UI libraries).

## Panic/Blocker Analysis: Samod Resolution

We previously encountered a panic with `samod` on WASM due to `SystemTime`.
**Resolution**: We have pivoted to a custom sync implementation based on
`todo_mvp`. This gives us full control over the stack and avoids blocking
upstream dependencies. The custom solution uses `automerge` directly and
standard WebSocket libraries (`gloo-net`, `axum`) which are known to work well
in their respective environments.

## Future Considerations

- **WASM Integration Testing**: Add `wasm-bindgen-test` integration tests for
  IndexedDB storage. Run with `wasm-pack test --headless --chrome` to test in a
  real browser environment without Playwright. This would provide lightweight
  WASM testing without the overhead of full E2E tests.
- **E2E Sync Testing**: Verify verified data consistency between two browser
  instances.

## Next Steps

1.  Implement **Milestone 2.2**: Sync Protocol (`sync-protocol`).
2.  Implement **Milestone 2.3**: Sync Server (`sync-server`).
3.  Implement **Milestone 2.4**: Store Refactor & Samod Removal.
