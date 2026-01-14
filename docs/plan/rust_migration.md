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
- **Sync:** Custom WebSocket Server (`tasklens-sync-server`) + Client
  `SyncService`.

## Subsystems & Isolation

We can identify three major layers that can be tackled somewhat independently:

1.  **Core Domain Logic (`tasklens-core`)**:
    - **Description**: The "brain" of the app. Priority sorting, recurrent task
      logic, visibility filters.
    - **Porting Strategy**: Direct port of `packages/tasklens/src/domain` types
      and functions to Rust.
    - **Verification**: Unit tests matching the logic of existing TS tests. Can
      reuse BDD YAML fixtures.

2.  **Persistence & Sync (`tasklens-store`, `tasklens-sync-protocol`,
    `tasklens-sync-server`)**:
    - **Description**: Handling Automerge documents, loading/saving to
      IndexedDB, syncing over WebSocket.
    - **Pivot Decision**: We are moving away from `samod` to a custom
      implementation (ported from `todo_mvp`) to avoid WASM incompatibility
      issues.
    - **Porting Strategy**:
      - `tasklens-sync-protocol`: Shared types for encryption and messaging.
      - `tasklens-sync-server`: Axum-based WebSocket server.
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
│   ├── tasklens-sync-protocol/      # [NEW] Shared sync types
│   └── tasklens-sync-server/        # [NEW] WebSocket server
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

- [x] **Milestone 2.2**: Sync Protocol (`tasklens-sync-protocol`).
  - **Goal**: Define shared types and protocol for syncing.
  - **Source**: `todo_mvp/sync_protocol`
  - **Implementation Details**:
    - [x] Create directory: `crates/tasklens-sync-protocol`
    - [x] Create `crates/tasklens-sync-protocol/Cargo.toml`:
      - [x] Add `[package]` section (name="tasklens-sync-protocol",
            version="0.1.0", edition="2021").
      - [x] Add `[dependencies]` section with
            `serde = { version = "1.0", features = ["derive"] }`.
    - [x] Create `crates/tasklens-sync-protocol/src/lib.rs`:
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

- [x] **Milestone 2.3**: Sync Server (`tasklens-sync-server`).
  - **Goal**: Implement the WebSocket sync server.
  - **Source**: `todo_mvp/sync_server`
  - **Implementation Details**:
    - **Step 1: Crate Setup**
      - [x] Create directory: `crates/tasklens-sync-server`
      - [x] Create `crates/tasklens-sync-server/Cargo.toml` (deps: axum, tokio,
            rusqlite, etc).
      - [x] Create `crates/tasklens-sync-server/src/main.rs` (skeleton).
    - **Step 2: Database**
      - [x] Create `crates/tasklens-sync-server/src/db.rs`.
      - [x] Implement `init_pool`, `append_update`, `get_changes_since`.
    - **Step 3: Server Logic**
      - [x] Implement `AppState` and `main` (setup router).
      - [x] Implement `ws_handler` and `handle_socket` (handshake + loop).
  - **Verification**:
    - [x] Run `cargo run -p tasklens-sync-server`.

- [x] **Milestone 2.3.5**: Rename & Align Sync Crates.
  - **Goal**: Rename crates to `tasklens-sync-protocol` and
    `tasklens-sync-server` and ensure 1:1 parity with `todo_mvp`.
  - **Details**:
    - [x] Rename `sync_protocols` -> `tasklens-sync-protocol`.
    - [x] Rename `sync_server` -> `tasklens-sync-server`.
    - [x] Diff against `todo_mvp` equivalents and revert gratuitous changes
          (keep implementations as close as possible).

- [x] **Milestone 2.4**: Store Refactor & Samod Removal (`tasklens-store`).
  - **Goal**: Remove `samod` and use `automerge` directly.
  - **Implementation Details**:
    - [x] **[MODIFY] `crates/tasklens-store/Cargo.toml`**:
      - Remove `samod` dependency.
      - Ensure `automerge`, `autosurgeon`, `unfold` are available.
    - [x] **[Refactor] `crates/tasklens-store/src/store.rs`**:
      - Replace `Repo` usage with `automerge::AutoCommit`.
      - Remove `start_sync`.
      - Implement `subscribe` using internal signals or polling.
    - [x] **[Cleanup] Workspace**:
      - Remove `samod` from root `Cargo.toml`.
  - **Verification**:
    - [x] `cargo check -p tasklens-store`.
    - [x] `cargo build --target wasm32-unknown-unknown -p tasklens-store`.
    - [x] All tests pass.

- [x] **Milestone 2.5**: Client Networking (`tasklens-store`).
  - **Goal**: Connect the client `Store` to the `tasklens-sync-server`.
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
    - [x] Add unit tests for `crypto.rs`.
    - [x] Integration test: Ensure `SyncService` can talk to a running
          `tasklens-sync-server`.

- [x] **Milestone 2.6**: Dioxus Integration ("Wholesale Port" Strategy).
  - **Goal**: Adopt the `todo_mvp` UI structure and connect the sync loop.
  - **Strategy**: Copy `todo_mvp` components 1:1 and adapt the main views to
    `tasklens` models. **STRICT REQUIREMENT** use the `cp` command to copy
    files, then edit them.
  - **Implementation Details**:
    - **1. Dependencies & Configuration**:
      - [x] **[MODIFY] `crates/tasklens-ui/Cargo.toml`**: Add `gloo-storage`,
            `bip39`, `argon2`, `chacha20poly1305`, `rand`, `hex`, `sha2`
            dependencies (referencing `todo_mvp/Cargo.toml` versions).
      - [x] **[MODIFY] `crates/tasklens-ui/Cargo.toml`**: Ensure
            `features = ["pwa"]` is present and aligned with `todo_mvp`.
      - [x] **[NEW] `crates/tasklens-ui/assets/`**: Create directory.
      - [x] **[NEW] `crates/tasklens-ui/public/`**: Create directory.
      - [x] **[CMD]**: Copy `todo_mvp/public/*` to `crates/tasklens-ui/public/`
            (PWA files).
      - [x] **[CMD]**: Copy `todo_mvp/assets/*` to `crates/tasklens-ui/assets/`
            (Shared assets).
      - [x] **[CMD]**: Copy `todo_mvp/tailwind.css` to
            `crates/tasklens-ui/tailwind.css` (Source).
      - [x] **[CMD]**: Copy `todo_mvp/assets/tailwind.css` to
            `crates/tasklens-ui/assets/tailwind.css` (Compiled output).
    - **2. Components (1:1 Copy)**:
      - [x] **[NEW] `crates/tasklens-ui/src/components/mod.rs`**: Copy from
            `todo_mvp/src/components/mod.rs`.
      - [x] **[NEW] `crates/tasklens-ui/src/components/alert.rs`**: Copy from
            `todo_mvp/src/components/alert.rs`.
      - [x] **[NEW] `crates/tasklens-ui/src/components/back_button.rs`**: Copy
            from `todo_mvp/src/components/back_button.rs`.
      - [x] **[NEW] `crates/tasklens-ui/src/components/button.rs`**: Copy from
            `todo_mvp/src/components/button.rs`.
      - [x] **[NEW] `crates/tasklens-ui/src/components/checkbox.rs`**: Copy from
            `todo_mvp/src/components/checkbox.rs`.
      - [x] **[NEW] `crates/tasklens-ui/src/components/input.rs`**: Copy from
            `todo_mvp/src/components/input.rs`.
      - [x] **[NEW] `crates/tasklens-ui/src/components/loading.rs`**: Copy from
            `todo_mvp/src/components/loading.rs`.
    - **3. Views: Authentication**:
      - [x] **[NEW] `crates/tasklens-ui/src/views/auth.rs`**: Copy from
            `todo_mvp/src/views/auth.rs`.
      - [x] **[MODIFY] `crates/tasklens-ui/src/views/auth.rs`**: Refactor
            `use crate::crypto` to `use tasklens_store::crypto`.
    - **4. Views: Task Page (The Adapter)**:
      - [x] **[NEW] `crates/tasklens-ui/src/views/mod.rs`**: Create module file
            exposing `auth` and `task_page`.
      - [x] **[NEW] `crates/tasklens-ui/src/views/task_page.rs`**: Port content
            from `todo_mvp/src/views/todo.rs`.
      - [x] **[MODIFY] `crates/tasklens-ui/src/views/task_page.rs`**: Rename
            component `TodoPage` -> `TaskPage`.
      - [x] **[MODIFY] `crates/tasklens-ui/src/views/task_page.rs`**: Update
            state imports: `crate::model::AppState` ->
            `tasklens_core::types::TunnelState`.
      - [x] **[MODIFY] `crates/tasklens-ui/src/views/task_page.rs`**: Update
            store imports: `crate::store::Store` ->
            `tasklens_store::store::AppStore`.
      - [x] **[MODIFY] `crates/tasklens-ui/src/views/task_page.rs`**: Update
            model imports: `crate::model::Todo` ->
            `tasklens_core::types::PersistedTask`.
      - [x] **[MODIFY] `crates/tasklens-ui/src/views/task_page.rs`**: Refactor
            `add_todo` to use
            `store.write().dispatch(Action::CreateTask { ... })`.
      - [x] **[MODIFY] `crates/tasklens-ui/src/views/task_page.rs`**: Refactor
            `toggle_todo` to use
            `store.write().dispatch(Action::UpdateTask { ... })`.
      - [x] **[MODIFY] `crates/tasklens-ui/src/views/task_page.rs`**: Update
            `TodoList` component to accept `HashMap<TaskID, PersistedTask>` (or
            convert to Vec) and render `PersistedTask` fields.
    - **5. Main Entry Point**:
      - [x] **[MODIFY] `crates/tasklens-ui/src/main.rs`**: Initialize
            `master_key` signal.
      - [x] **[MODIFY] `crates/tasklens-ui/src/main.rs`**: Add startup future to
            load key from storage (`tasklens_store::crypto::load_key()`).
      - [x] **[MODIFY] `crates/tasklens-ui/src/main.rs`**: Replace root
            component logic to render `TaskPage` (passing `master_key`).
      - [x] **[MODIFY] `crates/tasklens-ui/src/main.rs`**: (Optional) Add PWA
            initialization logic if using service worker.
  - **Verification**:
    - [x] `cargo check -p tasklens-ui` passes.
    - [x] `cargo build --target wasm32-unknown-unknown -p tasklens-ui` passes.
    - [x] `dx serve --package tasklens-ui`: Launch app, verify UI loads, and
          "Sync Active" indicator (green dot) appears when a key is
          generated/entered.

- [x] **Milestone 2.7**: Service Worker Integration (PWA).
  - **Goal**: Enable PWA capabilities matching `todo_mvp`.
  - **Implementation Details**:
    - [x] **[NEW] `crates/tasklens-ui/index.html`**: Copied from `todo_mvp` with
          updated title.
    - [x] **[VERIFY]**: Check `build.rs` generates `public/version.js` and
          `OUT_DIR/build_version.rs`.
    - [x] **[VERIFY]**: PWA installation available in browser (Localhost or via
          ngrok if needed).
    - [x] **[VERIFY]**: Service worker update cycle (Force update on reload).
    - [x] **[VERIFY] `crates/tasklens-ui/src/main.rs`**: Ensure
          `register_service_worker` is called and status is tracked. (Already
          present, verified behavior).

- [x] **Milestone 2.8**: Playwright Infrastructure & BDD Port.
  - **Goal**: Establish the E2E testing harness for Dioxus by porting BDD specs
    from the React app.
  - **Strategy**: Copy configuration and feature files verbatim, setting up the
    infrastructure to run them against the Dioxus app.
  - **Implementation Details**:
    - [x] **[MODIFY] `crates/tasklens-ui/package.json`**:
      - Add `playwright-bdd`, `@playwright/test`, and related dependencies.
      - Add scripts: `test-e2e`, `generate`.
    - [x] **[NEW] `crates/tasklens-ui/playwright.config.ts`**:
      - Port from `apps/client/playwright.config.ts`.
      - Configure `webServer` to run `dx serve` with a fixed port (e.g. `5180`).
      - Adjust paths to point to `tests/e2e`.
    - [x] **[NEW] `crates/tasklens-ui/tests/e2e/features/`**:
      - Copy all `.feature` files from `apps/client` verbatim.
    - [x] **[NEW] `crates/tasklens-ui/tests/e2e/steps/`**:
      - Copy `all.steps.ts` from `apps/client`.
    - [x] **[NEW] `crates/tasklens-ui/tests/e2e/fixtures.ts`**:
      - Copy from `apps/client`.
    - [x] **[NEW] `crates/tasklens-ui/tests/e2e/pages/`**:
      - Copy Page Objects from `apps/client`.
    - [x] **[VERIFY] Tool Configuration**:
      - Ensure `prettier`, `biome`, and `eslint` are configured to ignore
        `.features-gen`.
  - **Verification**:
    - [x] **Generate Tests**: Run `pnpm generate` and verify
          `tests/e2e/.features-gen` is populated with `.spec.ts` files.
    - [x] **Verify Harness**: Create a minimal "Smoke" feature that checks the
          app title or a static element to confirm the harness works.
    - [x] **[MODIFY] `crates/tasklens-ui/playwright.config.ts`**:
      - Configure `grepInvert: /@migration-pending/` to skip unimplemented
        features.
    - [x] **[MODIFY] Feature Files**:
      - Bulk add `@migration-pending` tag to all ported feature files (since
        functionality is not yet implemented).
  - **Verification**:
    - [x] **Generate Tests**: Run `pnpm generate` and verify
          `tests/e2e/.features-gen` is populated with `.spec.ts` files.
    - [x] **Verify Harness**: Create a minimal "Smoke" feature (tagged `@smoke`)
          that checks the app title to confirm the harness works.
    - [x] **Run Tests**: Execute `pnpm test-e2e` (MUST PASS: expecting 0
          failures due to `@migration-pending` exclusion).

### Epoch 3: Feature Parity (The Grind)

_Goal: Porting UI components to match functionality._

- [ ] **Milestone 3.1**: Foundation & "Plan" View.
  - **Goal**: Render a static hierarchy of tasks in the "Plan" view and support
    seeding sample data.
  - **Why**: "Plan" view is simpler (no priority sorting) and standardizes the
    data loading and rendering pipeline.
  - **Reference**: `prd.md` §5.1 "The Plan View".
  - **Dioxus Component Library Integration**:
    - **Policy**: Prefer standard components from the Dioxus component library
      over custom implementations.
    - **Installation**: Use `dx components add [name]`.
    - **Available Primitives**:
      - Layout: `Aspect Ratio`, `Card`, `Collapsible`, `Scroll Area`,
        `Separator`, `Sheet`
      - Navigation: `Menubar`, `NavBar`, `Tabs`, `Toolbar`
      - Form: `Button`, `Checkbox`, `Input`, `Label`, `Radio Group`, `Select`,
        `Slider`, `Switch`, `Textarea`, `Toggle`, `Toggle Group`
      - Feedback: `Alert Dialog`, `Dialog`, `Popover`, `Progress`, `Skeleton`,
        `Toast`, `Tooltip`
      - Other: `Accordion`, `Avatar`, `Calendar`, `Context Menu`, `Date Picker`,
        `Dropdown Menu`, `Hover Card`
      - _Missing_: `Badge` (Custom implementation required).
  - **Implementation Details**:
    - **Step 1: Router & Navigation Foundation**
      - [ ] **[MODIFY] `crates/tasklens-ui/src/views/mod.rs`**:
        - [ ] Export `plan_page`.
      - [ ] **[CMD] Install NavBar**: `dx components add navbar`.
      - [ ] **[MODIFY] `crates/tasklens-ui/src/components/navbar.rs`**:
        - [ ] Customize the installed `NavBar` to render links to `/plan`,
              `/do`, `/balance`.
        - [ ] Ensure it uses `Outlet` for child content.
      - [ ] **[MODIFY] `crates/tasklens-ui/src/main.rs`**:
        - [ ] Define `Route` enum with `#[derive(Routable)]`.
        - [ ] Add `#[route("/")]`, `#[route("/plan")]`, `#[route("/do")]`.
        - [ ] Wrap routes in `#[layout(NavBar)]`.
        - [ ] Update `App` to render `Router::<Route> {}`.

    - **Step 2: Seed Data (Test Fixtures)**
      - [ ] **[NEW] `crates/tasklens-ui/src/seed.rs`**:
        - [ ] Implement `prime_store_with_sample_data(store: &AppStore)`.
        - [ ] Create a "Root Project" task.
        - [ ] Create 3-5 child tasks (some with deeper nesting) to verify
              hierarchy.
        - [ ] Ensure `TaskID`s are valid UUIDs or deterministic strings for
              testing.
      - [ ] **[MODIFY] `crates/tasklens-ui/src/main.rs`**:
        - [ ] In the startup future, check
              `web_sys::window().location().search()` for `?seed=true`.
        - [ ] If present, await `prime_store_with_sample_data`.

    - **Step 3: UI Components**
      - [ ] **[CMD] Install Atoms**: `dx components add checkbox`.
      - [ ] **[NEW] `crates/tasklens-ui/src/components/task_row.rs`**:
        - [ ] Create `TaskRow` component.
        - [ ] **Props**:
          - `task: PersistedTask` (The data to show)
          - `depth: usize` (For indentation)
        - [ ] **Render**:
          - `div` container with `pl-[depth * 20px]`.
          - `Checkbox` (library component, disabled state).
          - `span` Task Title.
          - `div` debug text (small gray) showing `id`.

    - **Step 4: Plan View Implementation**
      - [ ] **[NEW] `crates/tasklens-ui/src/views/plan_page.rs`**:
        - [ ] Create `PlanPage` component.
        - [ ] **State Access**: Use `use_context::<Signal<AppStore>>()` (or
              `use_signal` depending on architecture).
        - [ ] **Tree Flattening**:
          - Implement logic to traverse `store.read().tasks` starting from
            `root_task_ids`.
          - Recursively maximize checking `child_task_ids`.
          - Produce a flat `Vec<(usize, PersistedTask)>` (depth, task).
        - [ ] **Render**: Loop over the flat list and render `TaskRow`.

  - **Verification**:
    - [ ] **[VERIFY]**: Navigate to `http://localhost:8080/?seed=true`.
    - [ ] **[VERIFY]**: Observe "Plan", "Do", "Balance" links in NavBar.
    - [ ] **[VERIFY]**: Click "Plan". Verify the hierarchy is rendered with
          correct indentation.
    - [ ] **[VERIFY]**: Verify `plan-management.feature` scenario "Render task
          hierarchy" (manual walkthroug or automated if harness allows).

- [ ] **Milestone 3.2**: Basic Mutation (Create & Toggle).
  - **Goal**: Allow users to create new tasks and toggle completion status.
  - **Implementation Details**:
    - [ ] **[MODIFY] `crates/tasklens-ui/src/views/plan_page.rs`**:
      - Add "Add Task" input (simple text input at top or bottom).
      - Dispatch `Action::CreateTask`.
    - [ ] **[MODIFY] `crates/tasklens-ui/src/components/task_row.rs`**:
      - Wire up Checkbox `onclick` to dispatch `Action::UpdateTask(status)`.
  - **Verification**:
    - [ ] **[VERIFY]**: Pass `task-creation.feature` (Basic creation).
    - [ ] **[VERIFY]**: Pass `task-lifecycle.feature` (Completion).

- [ ] **Milestone 3.3**: The "Do" View (Priority List).
  - **Goal**: Implement the algorithmic priority list ("Do" View).
  - **Implementation Details**:
    - [ ] **[NEW] `crates/tasklens-ui/src/views/do_page.rs`**:
      - Use `tasklens_core::get_prioritized_tasks` to sort.
      - Render flat list of `TaskRow`s.
    - [ ] **[MODIFY] `crates/tasklens-ui/src/components/task_row.rs`**:
      - Add visual cues: Due Dates, Urgency Coloring (Red/Orange/Green), Project
        path breadcrumbs.
  - **Verification**:
    - [ ] **[VERIFY]**: Pass `routine-tasks.feature` (Appearance in Do view).
    - [ ] **[VERIFY]**: Pass `due-dates.feature` (Sorting & Badges).

- [ ] **Milestone 3.4**: Task Details & Editing.
  - **Goal**: Full task modification (Dates, Notes, Lead Time, Repetition).
  - **Implementation Details**:
    - [ ] **[NEW] `crates/tasklens-ui/src/components/task_editor.rs`**:
      - Modal or Slide-over panel.
      - Fields: Title, Notes, Due Date (Date Picker), Lead Time (Duration
        Parser), Repetition.
  - **Verification**:
    - [ ] **[VERIFY]**: Pass `task-creation.feature` (Defaults, Child
          inheritance).

- [ ] **Milestone 3.5**: Drag & Drop Reordering.
- [ ] **Milestone 3.6**: Settings & Mobile Polish.

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

1.  Implement **Milestone 3.1**: Task List & Inspection.
2.  Begin un-tagging features from `@migration-pending` as implementation
    progresses.
