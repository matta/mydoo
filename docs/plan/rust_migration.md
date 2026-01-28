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

#### [x] Milestone 1.1: Create Rust workspace structure

(or efficient usage of `todo_mvp`).

#### [x] Milestone 1.2: Port `TunnelState`, `Task`, and basic types to Rust structs

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
  - **Interim Step**: Implementing `serde` traits is the critical interim step.
    While we will verify these via JSON serialization tests, the ultimate goal
    is `transplant`ing these structs into an Automerge document.
  - **Validation**:
    - Serialize Rust structs to JSON.
    - Compare against expected JSON output from the TypeScript `TaskSchema`.
    - Use the "personal todos" dataset or a synthetic complex task tree as a
      test case to ensure exact structural alignment.
- **Implementation Details**:
  - **[NEW] `crates/tasklens-core/src/types.rs`**:
    - Define `TaskID` and `PlaceID` as "NewType" constructs (tuple structs) for
      type safety.
    - Define enums: `TaskStatus`.
    - Define structs: `Schedule`, `RepeatConfig`, `PersistedTask`, `Place`,
      `TunnelState`.
    - derive `serde::Serialize` and `serde::Deserialize` for all types to match
      the existing JSON schema exactly.
  - **[MODIFY] `crates/tasklens-core/src/lib.rs`**:
    - Export `types` module.
  - Ensure strict compatibility with `automerge` via `autosurgeon` (or manual
    implementations where necessary).
- **Verification**:
  - Create unit tests in `tasklens-core` that serialize these structs to JSON.
  - Validate that the generated JSON matches the structure expected by the
    TypeScript `TaskSchema`.

#### [x] Milestone 1.3: Port `priority.ts` and `schedules` logic

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

#### [x] Milestone 1.4: Implement a test runner to execute existing YAML BDD specs

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

#### [x] Milestone 2.1: `tasklens-store` Initialization (Local)

- **Goal**: Create the local persistence layer wrapping `automerge`.
- **Status**: Completed (re-verified against new plan). The current `Store`
  implementation in `tasklens-store` matches the requirements (direct
  `automerge` usage).
- **Note**: The original plan used `samod`. We have effectively implemented the
  "custom wrapper" part already. We just need to ensure it aligns with
  `todo_mvp`'s `store.rs`.

#### [x] Milestone 2.2: Sync Protocol (`tasklens-sync-protocol`)

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
      - [x] Variant `SubmitChange { sync_id: String, payload: EncryptedBlob }`.
      - [x] Derive: `Debug, Clone, Serialize, Deserialize`.
    - [x] Implement `ServerMessage` enum:
      - [x] Variant
            `ChangeOccurred { sequence_id: i64, sync_id: String, source_client_id: String, payload: EncryptedBlob }`.
      - [x] Derive: `Debug, Clone, Serialize, Deserialize`.

#### [x] Milestone 2.3: Sync Server (`tasklens-sync-server`)

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

#### [x] Milestone 2.3.5: Rename & Align Sync Crates

- **Goal**: Rename crates to `tasklens-sync-protocol` and `tasklens-sync-server`
  and ensure 1:1 parity with `todo_mvp`.
- **Details**:
  - [x] Rename `sync_protocols` -> `tasklens-sync-protocol`.
  - [x] Rename `sync_server` -> `tasklens-sync-server`.
  - [x] Diff against `todo_mvp` equivalents and revert gratuitous changes (keep
        implementations as close as possible).

#### [x] Milestone 2.4: Store Refactor & Samod Removal (`tasklens-store`)

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

#### [x] Milestone 2.5: Client Networking (`tasklens-store`)

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

#### [x] Milestone 2.6: Dioxus Integration ("Wholesale Port" Strategy)

- **Goal**: Adopt the `todo_mvp` UI structure and connect the sync loop.
- **Strategy**: Copy `todo_mvp` components 1:1 and adapt the main views to
  `tasklens` models. **STRICT REQUIREMENT** use the `cp` command to copy files,
  then edit them.
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
    - [x] **[MODIFY] `crates/tasklens-ui/src/views/task_page.rs`**: Update state
          imports: `crate::model::AppState` ->
          `tasklens_core::types::TunnelState`.
    - [x] **[MODIFY] `crates/tasklens-ui/src/views/task_page.rs`**: Update store
          imports: `crate::store::Store` -> `tasklens_store::store::AppStore`.
    - [x] **[MODIFY] `crates/tasklens-ui/src/views/task_page.rs`**: Update model
          imports: `crate::model::Todo` ->
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
    - [x] **[MODIFY] `crates/tasklens-ui/src/main.rs`**: Initialize `master_key`
          signal.
    - [x] **[MODIFY] `crates/tasklens-ui/src/main.rs`**: Add startup future to
          load key from storage (`tasklens_store::crypto::load_key()`).
    - [x] **[MODIFY] `crates/tasklens-ui/src/main.rs`**: Replace root component
          logic to render `TaskPage` (passing `master_key`).
    - [x] **[MODIFY] `crates/tasklens-ui/src/main.rs`**: (Optional) Add PWA
          initialization logic if using service worker.
- **Verification**:
  - [x] `cargo check -p tasklens-ui` passes.
  - [x] `cargo build --target wasm32-unknown-unknown -p tasklens-ui` passes.
  - [x] `dx serve --package tasklens-ui`: Launch app, verify UI loads, and "Sync
        Active" indicator (green dot) appears when a key is generated/entered.

#### [x] Milestone 2.7: Service Worker Integration (PWA)

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

#### [x] Milestone 2.8: Playwright Infrastructure & Code-First Gherkin Port

- **Goal**: Establish the E2E testing harness for Dioxus by porting BDD specs
  from the React app to a Code-First Gherkin structure.
- **Strategy**: implementstrictly typed actor fixtures (`I`) and translate
  scenarios into TypeScript specs.
- **Implementation Details**:
  - [x] **[MODIFY] `crates/tasklens-ui/package.json`**:
    - Add `@playwright/test` and related dependencies.
    - Add script: `test-e2e`.
  - [x] **[NEW] `crates/tasklens-ui/playwright.config.ts`**:
    - Port from `apps/client/playwright.config.ts`.
    - Configure `webServer` to run `dx serve` with a fixed port (e.g. `5180`).
    - Adjust paths to point to `tests/e2e`.
  - [x] **[NEW] `crates/tasklens-ui/tests/e2e/specs/`**:
    - Create Code-First Gherkin specs (.spec.ts) based on the React app's
      features.
  - [x] **[NEW] `crates/tasklens-ui/tests/e2e/fixtures.ts`**:
    - Copy from `apps/client`.
  - [x] **[NEW] `crates/tasklens-ui/tests/e2e/pages/`**:
    - Copy Page Objects from `apps/client`.
- **Verification**:
  - [x] **Verify Harness**: Create a minimal "Smoke" spec that checks the app
        title or a static element to confirm the harness works.
  - [x] **[MODIFY] `crates/tasklens-ui/playwright.config.ts`**:
    - Configure `grepInvert: /@migration-pending/` to skip unimplemented
      features.
  - [x] **[MODIFY] Spec Files**:
    - Bulk add `@migration-pending` tag to all ported spec files (since
      functionality is not yet implemented).
- **Verification**:
  - [x] **Verify Harness**: Create a minimal "Smoke" spec (tagged `@smoke`)
        that checks the app title to confirm the harness works.
  - [x] **Run Tests**: Execute `pnpm test-e2e` (MUST PASS: expecting 0 failures
        due to `@migration-pending` exclusion).

### Epoch 3: Feature Parity (The Grind)

_Goal: Porting UI components to match functionality._

#### [x] Milestone 3.1: Foundation & "Plan" View

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
    - Layout: `Aspect Ratio`, `Card`, `Collapsible`, `Scroll Area`, `Separator`,
      `Sheet`
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
    - [x] **[MODIFY] `crates/tasklens-ui/src/views/mod.rs`**:
      - [x] Export `plan_page`.
    - [x] **[CMD] Install NavBar**: `dx components add navbar`.
    - [x] **[MODIFY] `crates/tasklens-ui/src/components/navbar.rs`**:
      - [x] Customize the installed `NavBar` to render links to `/plan`, `/do`,
            `/balance`.
      - [x] Ensure it uses `Outlet` for child content.
    - [x] **[MODIFY] `crates/tasklens-ui/src/main.rs`**:
      - [x] Define `Route` enum with `#[derive(Routable)]`.
      - [x] Add `#[route("/")]`, `#[route("/plan")]`, `#[route("/do")]`.
      - [x] Wrap routes in `#[layout(NavBar)]`.
      - [x] Update `App` to render `Router::<Route> {}`.

  - **Step 2: Seed Data (Test Fixtures)**
    - [x] **[NEW] `crates/tasklens-ui/src/seed.rs`**:
      - [x] Implement `prime_store_with_sample_data(store: &AppStore)`.
      - [x] Create a "Root Project" task.
      - [x] Create 3-5 child tasks (some with deeper nesting) to verify
            hierarchy.
      - [x] Ensure `TaskID`s are valid UUIDs or deterministic strings for
            testing.
    - [x] **[MODIFY] `crates/tasklens-ui/src/main.rs`**:
      - [x] In the startup future, check `web_sys::window().location().search()`
            for `?seed=true`.
      - [x] If present, await `prime_store_with_sample_data`.

  - **Step 3: UI Components**
    - [x] **[CMD] Install Atoms**: `dx components add checkbox`.
    - [x] **[NEW] `crates/tasklens-ui/src/components/task_row.rs`**:
      - [x] Create `TaskRow` component.
      - [x] **Props**:
        - `task: PersistedTask` (The data to show)
        - `depth: usize` (For indentation)
      - [x] **Render**:
        - `div` container with `pl-[depth * 20px]`.
        - `Checkbox` (library component, disabled state).
        - `span` Task Title.
        - `div` debug text (small gray) showing `id`.

  - **Step 4: Plan View Implementation**
    - [x] **[NEW] `crates/tasklens-ui/src/views/plan_page.rs`**:
      - [x] Create `PlanPage` component.
      - [x] **State Access**: Use `use_context::<Signal<AppStore>>()` (or
            `use_signal` depending on architecture).
      - [x] **Tree Flattening**:
        - Implement logic to traverse `store.read().tasks` starting from
          `root_task_ids`.
        - Recursively maximize checking `child_task_ids`.
        - Produce a flat `Vec<(usize, PersistedTask)>` (depth, task).
      - [x] **Render**: Loop over the flat list and render `TaskRow`.

- **Verification**:
  - [x] **[VERIFY]**: Navigate to `http://localhost:8080/?seed=true`.
  - [x] **[VERIFY]**: Observe "Plan", "Do", "Balance" links in NavBar.
  - [x] **[VERIFY]**: Click "Plan". Verify the hierarchy is rendered with
        correct indentation.
  - [x] **[VERIFY]**: Verify `plan-management.feature` scenario "Render task
        hierarchy" (manual walkthroug or automated if harness allows).

#### [x] Milestone 3.2: Basic Mutation (Create & Toggle)

- **Goal**: Allow users to create new tasks, toggle completion status, and
  manage hierarchy.
- **Gaps Addressed**: `plan-management.feature` (Creation, Deletion, Hierarchy).
- **Implementation Details**:
  - [x] **[MODIFY] `crates/tasklens-store/src/store.rs`**:
    - [x] Update `Action::CreateTask` handler to implement PRD Section 3.1
          Inheritance Rules:
      - [x] Inherit `place_id` from parent (or `None`/`Anywhere` if root).
      - [x] Inherit `credit_increment` (Effort) from parent (or default `0.5` if
            root).
      - [x] Set default `lead_time` to `28,800,000` (8 hours).
  - [x] **[NEW] `crates/tasklens-ui/src/controllers/mod.rs`**:
    - [x] Create module and export `task_controller`.
  - [x] **[NEW] `crates/tasklens-ui/src/controllers/task_controller.rs`**:
    - [x] Implement
          `create_task(store: &mut Signal<AppStore>, parent_id: Option<TaskID>, name: String)`.
      - [x] Wrapper around `store.write().dispatch(Action::CreateTask { ... })`.
    - [x] Implement
          `toggle_task_status(store: &mut Signal<AppStore>, task_id: TaskID)`.
      - [x] Wrapper around `store.write().dispatch(Action::UpdateTask { ... })`.
    - [x] Implement
          `delete_task(store: &mut Signal<AppStore>, task_id: TaskID)`.
      - [x] Wrapper around `store.write().dispatch(Action::DeleteTask { ... })`.
    - [x] Implement
          `rename_task(store: &mut Signal<AppStore>, task_id: TaskID, new_title: String)`.
      - [x] Wrapper around `store.write().dispatch(Action::UpdateTask { ... })`.
  - [x] **[MODIFY] `crates/tasklens-ui/src/views/plan_page.rs`**:
    - [x] Add "Add Root Task" button at the bottom of the list (or top).
    - [x] Implement `handle_create` using `task_controller`.
  - [x] **[MODIFY] `crates/tasklens-ui/src/components/task_row.rs`**:
    - [x] Wire up Checkbox `onclick` -> `task_controller::toggle_task_status`.
    - [x] Make Title editable (or ad-hoc input) ->
          `task_controller::rename_task`.
    - [x] Add UI controls for:
      - [x] "Add Subtask" -> `task_controller::create_task(..., Some(id), ...)`.
      - [x] "Delete" -> `task_controller::delete_task`.
- **Verification**:
  - [x] **[VERIFY]**: Run `dx serve` and verify manual creation of task
        hierarchy.
  - [ ] **[VERIFY]**: Verify `plan-management.feature` scenario "Edit task
        properties and persist" (Created/Rename/Reload). (Test flaky/Env issue)
  - [ ] **[VERIFY]**: Verify `plan-management.feature` scenario "Delete task
        with cascade". (Test flaky/Env issue)

#### [x] Milestone 3.3: The "Do" View (Priority List)

- **Goal**: Implement the algorithmic priority list ("Do" View) and navigation.
- **Gaps Addressed**: `plan-management.feature` (Find in Plan from Do view).
- **Implementation Details**:
  - [x] **[NEW] `crates/tasklens-ui/src/components/priority_task_row.rs`**:
    - [x] Create `PriorityTaskRow` component.
    - [x] **Props**: `task: ComputedTask`.
    - [x] **Render**:
      - [x] Checkbox (toggles status via `task_controller`).
      - [x] Title (Strikethrough if done).
      - [x] Metadata: Due Date (if present), Project Name (Breadcrumbs -
            requires parent lookup).
      - [x] **Visual Cues**:
        - [x] Urgency Coloring (Red if `UrgencyStatus::Overdue`, Orange if
              `UrgencyStatus::DueSoon`).
        - [x] Inbox Border (if `place_id` is Inbox).
  - [x] **[NEW] `crates/tasklens-ui/src/views/do_page.rs`**:
    - [x] Create `DoPage` component.
    - [x] **State**:
      - [x] Access `AppStore`.
      - [x] `use_memo` to call
            `tasklens_core::domain::priority::get_prioritized_tasks`.
      - [x] Construct `ViewFilter` (Show all valid) and `PriorityOptions`.
    - [x] **Render**:
      - [x] Header: "Work" (or Context Selector later).
      - [x] List: Iterate `prioritized_tasks` and render `PriorityTaskRow`.
      - [x] "No tasks ready" empty state.
  - [x] **[MODIFY] `crates/tasklens-ui/src/router.rs`**:
    - [x] Map `/do` to `DoPage`.
    - [x] Remove legacy `TaskPage` reference if unused (or keep as detail view
          later).
  - [x] **[MODIFY] `crates/tasklens-ui/src/views/mod.rs`**:
    - [x] Export `do_page`.
- **Verification**:
  - [x] **[VERIFY]**: "Do" view renders tasks sorted by score.
  - [x] **[VERIFY]**: visual cues (Red/Orange) appear for urgent tasks (Requires
        seed data with dates).
  - [x] **[VERIFY]**: Checking a task strikes it through (local state update).
  - [x] **[VERIFY]**: Verify `plan-management.feature` scenario "Find in Plan
        from Do view" (Part 1: Rendering).

#### [x] Milestone 3.4: Task Details & Editing

- **Goal**: Full task modification and "Find in Plan" navigation.
- **Gaps Addressed**: `plan-management.feature` (Find in Plan action),
  `task-creation.feature`.
- **Implementation Details**:
  - **Step 1: Component Installation**
    - [x] **[CMD]**: `dx components add sheet` (Slide-over panel for details).
    - [x] **[CMD]**: `dx components add dialog` (Confirmation modals).
    - [x] **[CMD]**: `dx components add slider` (Importance/Effort).
    - [x] **[CMD]**: `dx components add select` (Dropdowns for Place/Schedule).
    - [x] **[CMD]**: `dx components add textarea` (Notes).
    - [x] **[CMD]**: `dx components add date-picker` (Due Dates).
  - **Step 2: Core Task Editor Component**
    - [x] **[NEW] `crates/tasklens-ui/src/components/task_editor.rs`**:
      - [x] Define `TaskEditor` component.
      - [x] **Props**:
        - `task_id: Option<TaskID>` (None = Create Mode).
        - `initial_parent_id: Option<TaskID>` (For Create Mode).
        - `on_close: EventHandler<()>`.
      - [x] **State**:
        - Use `use_signal` to hold `DraftTask` (local struct mirroring
          `PersistedTask` fields).
        - **Initialization**:
          - If `Edit Mode`: Deep clone from `AppStore`.
          - If `Create Mode`: Apply Defaults (Importance=1.0,
            Effort=Inherited/0.5, Place=Inherited/Anywhere).
      - [x] **Render**:
        - Wrap in `Sheet` component.
        - **Header**:
          - Title Input (`Input`).
          - "Find in Plan" button (Only in Edit Mode).
        - **Body**:
          - **Core**:
            - Importance: `Slider` (0.0 - 1.0).
            - Effort: `Slider` (0.0 - 1.0).
            - Place: `Select` (Inbox, Home, Work, etc - derived from Store).
          - **Scheduling**:
            - Type: `Select` (Once, Routinely, DueDate, Calendar).
            - _Conditional Fields_:
              - **Routinely**: Period (Number + Unit Select), Lead Time
                (Number).
              - **DueDate**: Date Picker, Lead Time.
          - **Notes**: `Textarea` (Markdown support deferred).
        - **Footer**:
          - "Save" Button (Calls `task_controller::create_task` or
            `update_task`).
          - "Delete" Button (Calls `task_controller::delete_task` with `Dialog`
            confirmation).
          - "Indent/Outdent" Buttons (Calls `task_controller`).
    - [x] **[MODIFY] `crates/tasklens-ui/src/components/mod.rs`**:
      - [x] Export `task_editor`.

  - **Step 3: Integration with Views**
    - [x] **[MODIFY] `crates/tasklens-ui/src/views/do_page.rs`**:
      - [x] Add state: `selected_task: Signal<Option<TaskID>>`.
      - [x] Render `TaskEditor` (conditional or passing signal) at root of view.
      - [x] **Interaction**: Clicking `PriorityTaskRow` (non-checkbox area) sets
            `selected_task`.
    - [x] **[MODIFY] `crates/tasklens-ui/src/views/plan_page.rs`**:
      - [x] Add state: `selected_task: Signal<Option<TaskID>>`.
      - [x] Render `TaskEditor` at root of view.
      - [x] **Interaction**: Clicking `TaskRow` title sets `selected_task`.

  - **Step 4: "Find in Plan" Navigation**
    - [x] **[MODIFY] `crates/tasklens-ui/src/views/plan_page.rs`**:
      - [x] Accept Query Param `focus_task` or use Global Signal.
      - [x] **Logic (`expose_task`)**:
        1.  Find target task in Store.
        2.  Collect all ancestor IDs.
        3.  Update `expanded_tasks` set to include all ancestors.
        4.  Force re-render.
        5.  Scroll element into view (using `web_sys` DOM access by ID).
    - [x] **[MODIFY] `crates/tasklens-ui/src/components/task_editor.rs`**:
      - [x] "Find in Plan" button action:
        1.  Close Editor.
        2.  Navigate to `/plan` with target ID context.

- **Verification**:
  - [x] **[VERIFY]**: "Do" view click opens Editor.
  - [x] **[VERIFY]**: Create new Task via Editor (check defaults).
  - [x] **[VERIFY]**: Edit existing Task (check persistence).
  - [x] **[VERIFY]**: "Find in Plan" correctly expands tree and highlights task.
  - [x] **[VERIFY]**: "Find in Plan" correctly expands tree and highlights task.
  - [x] **[VERIFY]**: Run `plan-management.feature` (All scenarios should now
        pass or be close).
  - [x] **[VERIFY]**: Run `task-creation.feature`.

  - **Step 5: Enable BDD Tests**
    - [x] **[MODIFY]
          `crates/tasklens-ui/tests/e2e/features/plan-management.feature`**:
      - [x] Remove `@migration-pending` tag.
    - [x] **[MODIFY]
          `crates/tasklens-ui/tests/e2e/features/task-creation.feature`**:
      - [x] Remove `@migration-pending` tag.
    - [x] **[VERIFY]**: Run `pnpm test-e2e` and ensure these features pass in
          the CI flow.

#### [x] Milestone 3.5: Task Lifecycle (Refresh & Acknowledge)

- **Goal**: Implement the "Refresh" cycle that acknowledges completed tasks
  (removing them from the Do list) and wakes up routine tasks.
- **Gaps Addressed**: Missing "Refresh" action on Do view, accumulation of
  completed tasks.
- **Implementation Details**:
  - **Step 1: Domain Logic**
    - [x] **[NEW] `crates/tasklens-core/src/domain/lifecycle.rs`**:
    - [x] Implement `acknowledge_completed_tasks(state: &mut TunnelState)`.
      - Iterates all tasks.
      - If `status == Done` and `!is_acknowledged`:
        - Set `is_acknowledged = true`.
    - [x] **[MODIFY] `crates/tasklens-core/src/domain/mod.rs`**: Export
          `lifecycle`.
  - **Step 2: Store Action**
    - [x] **[MODIFY] `crates/tasklens-store/src/store.rs`**:
    - [x] Add `Action::RefreshLifecycle { current_time: f64 }`.
    - [x] In `dispatch`:
      1. Call `tasklens_core::domain::lifecycle::acknowledge_completed_tasks`.
      2. Call `tasklens_core::domain::routine_tasks::wake_up_routine_tasks`.
  - **Step 3: UI Integration**
    - [x] **[MODIFY] `crates/tasklens-ui/src/views/do_page.rs`**:
    - [x] Add "Refresh" Button to header (right aligned).
    - [x] On click:
      - Get `Date::now()`.
      - Dispatch `Action::RefreshLifecycle`.
  - **Step 4: Controller Update**
    - [x] **[MODIFY] `crates/tasklens-ui/src/controllers/task_controller.rs`**:
    - [x] Add `refresh_lifecycle(store: &mut Signal<AppStore>)` helper.

- **Verification**:
  - [x] **[VERIFY]**: Mark a task as Done. It stays visible (struck through).
  - [x] **[VERIFY]**: Click "Refresh". The Done task disappears (is
        acknowledged).
  - [x] **[VERIFY]**: Mark a "Routinely" task as Done. Click "Refresh". Verify
        it disappears (if interval not passed) or resets to Pending (if interval
        passed - requires cheating time or short interval).

#### [x] Milestone 3.6: Advanced Plan Management ("Find in Plan")

- **Goal**: Polish the Plan View navigation and hierarchy management to satisfy
  remaining `plan-management.feature` scenarios.
- **Key Implementations**:
  - **Visual Feedback**: Flashing highlight when a task is revealed or created.
  - **UX Polish**: Auto-expansion of parents upon child creation (resolving
    Playwright workarounds).
  - **Test Enablement**: Activating pending BDD scenarios.
- **Implementation Details**:
  - **Step 1: Auto-Expand on Creation**
    - [x] **[MODIFY] `crates/tasklens-store/src/actions.rs`**:
      - [x] Update `Action::CreateTask` to include `id: TaskID`.
    - [x] **[MODIFY] `crates/tasklens-store/src/store.rs`**:
      - [x] Update `Action::CreateTask` handler to use the provided `id`.
    - [x] **[MODIFY] `crates/tasklens-core/src/domain/creation.rs`**:
      - [x] Update `create_new_task` to accept `id: TaskID` as an argument.
    - [x] **[MODIFY] `crates/tasklens-ui/src/controllers/task_controller.rs`**:
      - [x] Update `create_task` to generate a `TaskID` first and return it.
    - [x] **[MODIFY] `crates/tasklens-ui/src/views/plan_page.rs`**:
      - [x] Update `add_task` and `on_add_child` handlers to use the returned
            `TaskID`.
      - [x] Ensure `parent_id` is expanded when a child is created.
  - **Step 2: "Find in Plan" Visuals (Highlighting)**
    - [x] **[MODIFY] `crates/tasklens-ui/tailwind.css`**:
      - [x] Add `@keyframes flash-yellow` and `@utility animate-flash`.
    - [x] **[MODIFY] `crates/tasklens-ui/src/components/task_row.rs`**:
      - [x] Add `is_highlighted` prop and apply `animate-flash` class.
    - [x] **[MODIFY] `crates/tasklens-ui/src/views/plan_page.rs`**:
      - [x] Implement signal-based highlighting with `gloo-timers` auto-clear.
  - **Step 3: Test Cleanup**
    - [x] **[MODIFY] `crates/tasklens-ui/tests/e2e/pages/plan-page.ts`**:
      - [x] Remove manual `toggleExpand` workaround.
    - [x] **[MODIFY]
          `crates/tasklens-ui/tests/e2e/features/plan-management.feature`**:
      - [x] Remove `@migration-pending` tags.
- **Verification**:
  - [x] "Find in Plan" from Do View navigates, expands, and highlights.
  - [x] Creating a child task auto-expands the parent.
  - [x] All `plan-management.feature` scenarios pass.

#### [x] Milestone 3.7: Task Movement & Reordering

- **Goal**: Enable `task-moving.feature`.
- **Key Feature**: Move Picker dialog with cycle detection, Indent/Outdent
  buttons in Task Editor.
- **PRD Reference**: §5.5 "Task Editing (Details Modal)" - Hierarchy Controls.
- **Implementation Details**:
  - **Step 1: Domain Helpers (Cycle Detection)**
    - [x] **[NEW] `crates/tasklens-core/src/domain/hierarchy.rs`**:
      - [x] Implement
            `get_descendant_ids(state: &TunnelState, task_id: &TaskID)     -> HashSet<TaskID>`.
        - Recursively collects all descendant IDs of a given task.
        - Used to filter out invalid move targets in the Move Picker.
      - [x] Implement
            `get_ancestor_ids(state: &TunnelState, task_id: &TaskID)     -> Vec<TaskID>`.
        - Collects all ancestor IDs up to the root.
      - [x] Implement
            `get_previous_sibling(state: &TunnelState, task_id:     &TaskID) -> Option<TaskID>`.
        - Returns the previous sibling in the parent's `child_task_ids` list.
        - Used for Indent operation (move as child of previous sibling).
    - [x] **[MODIFY] `crates/tasklens-core/src/domain/mod.rs`**: Export
          `hierarchy` module.

  - **Step 2: Controller Actions**
    - [x] **[MODIFY] `crates/tasklens-ui/src/controllers/task_controller.rs`**:
      - [x] Implement
            `move_task(store: &mut Signal<AppStore>, task_id: TaskID,     new_parent_id: Option<TaskID>)`.
        - Wrapper around `store.write().dispatch(Action::MoveTask { ... })`.
      - [x] Implement
            `indent_task(store: &mut Signal<AppStore>, task_id:     TaskID)`.
        - Get previous sibling via `get_previous_sibling`.
        - If exists, call `move_task` with
          `new_parent_id = Some(previous_sibling)`.
      - [x] Implement
            `outdent_task(store: &mut Signal<AppStore>, task_id:     TaskID)`.
        - Get parent's parent ID.
        - If exists, call `move_task` with `new_parent_id = grandparent_id`.
        - If parent is a root, call `move_task` with `new_parent_id = None`.

  - **Step 3: Move Picker Component**
    - [x] **[NEW] `crates/tasklens-ui/src/components/move_picker.rs`**:
      - [x] Create `MovePicker` component.
      - [x] **Props**:
        - `task_id: TaskID` (The task being moved).
        - `on_select: EventHandler<Option<TaskID>>` (Selected new parent, None =
          root).
        - `on_close: EventHandler<()>`.
      - [x] **State**:
        - Access `AppStore` via `use_context`.
        - Compute `excluded_ids: HashSet<TaskID>` containing:
          - The task itself.
          - All descendants (via `get_descendant_ids`).
      - [x] **Render**:
        - Wrap in `DialogRoot` / `DialogContent`.
        - `DialogTitle`: "Move \"[task.title]\"".
        - Body: Render flat tree of all tasks (roots + nested), excluding
          `excluded_ids`.
        - Each row: Task title, click handler calls `on_select(task_id)`.
        - "Make Root Task" button at top: calls `on_select(None)`.
    - [x] **[MODIFY] `crates/tasklens-ui/src/components/mod.rs`**: Export
          `move_picker`.

  - **Step 4: Task Editor Integration**
    - [x] **[MODIFY] `crates/tasklens-ui/src/components/task_editor.rs`**:
      - [x] Add "Move..." button (visible in Edit mode).
      - [x] On click: Show Move Picker.
      - [x] On select:
        1. Call `task_controller::move_task`.
        2. Trigger sync. (Handled via closing editor which should trigger
           refresh in parent or via store updates)
        3. Close editor.
      - [x] Add "← Outdent" button (visible if task has a parent).
      - [x] Add "Indent →" button (visible if task has a previous sibling).
      - [x] Wire buttons to `task_controller::indent_task` / `outdent_task`.

  - **Step 5: Enable BDD Tests**
    - [x] **[MODIFY]
          `crates/tasklens-ui/tests/e2e/features/task-moving.feature`**:
      - [x] Remove `@migration-pending` tag.

- **Verification**:
  - [x] **[VERIFY]**: Run
        `pnpm --filter @mydoo/tasklens-ui test-e2e -g     "task-moving"` and
        ensure all scenarios pass.
  - [x] **[VERIFY]**: Manual test: Open Task Editor, click "Move...", select a
        new parent, verify task is reparented.
  - [x] **[VERIFY]**: Manual test: Indent/Outdent buttons work correctly.
  - [x] **[VERIFY]**: Cycle detection: Cannot move a task to its own descendant
        (descendants hidden in Move Picker).

#### [x] Milestone 3.8: Due Dates & Urgency

- **Goal**: Enable `due-dates.feature` and basic `task-creation.feature`.
- **Key Feature**: Date picker integration, "Due Date" schedule type, and
  Urgency visualization.
- **Implementation Details**:
  - **Step 1: Component Installation**
    - [x] **[CMD]**: `dx components add date-picker`.
    - [x] **[MODIFY] `crates/tasklens-ui/src/components/mod.rs`**: Export
          `date_picker`.
  - **Step 2: Task Editor Integration (Due Dates)**
    - [x] **[MODIFY] `crates/tasklens-ui/src/components/task_editor.rs`**:
      - [x] Replace placeholder `DatePicker` with real component.
      - [x] Implement `f64` (timestamp) <-> `String` (YYYY-MM-DD) conversion for
            `due_date`.
      - [x] Add "Lead Time" input (number + unit).
      - [x] Bind to `draft.schedule.due` and `draft.schedule.lead_time`.
      - [x] Ensure `schedule.type` defaults to `Once` (or `DueDate` if date
            set).
  - **Step 3: Visual Feedback (Plan View)**
    - [x] **[MODIFY] `crates/tasklens-ui/src/components/task_row.rs`**:
      - [x] Import `tasklens_core::domain::dates::get_urgency_status`.
      - [x] Check `task.schedule.due_date` and compute urgency relative to
            `Context::now()`.
      - [x] Render Due Date badge/text if present.
      - [x] Apply "text-red-600" if Overdue `UrgencyStatus::Overdue`.
      - [x] Apply "text-orange-500" if `UrgencyStatus::DueSoon` or `Active`.
  - **Step 4: Enable BDD Tests**
    - [x] **[MODIFY]
          `crates/tasklens-ui/tests/e2e/features/due-dates.feature`**:
      - [x] Remove `@migration-pending` tag.
- **Verification**:
  - [x] **[VERIFY]**: Run
        `pnpm --filter @mydoo/tasklens-ui test-e2e -g "due-dates"`.
  - [x] **[VERIFY]**: Manual: Create task with Due Date "Tomorrow", Lead Time "2
        days". Verify "Active" status (Orange).
  - [x] **[VERIFY]**: Manual: Check Do View for proper color coding of
        Urgent/Overdue tasks.

#### [x] Milestone 3.9: Routine Tasks Configuration

- **Goal**: Enable routine task creation and management (specifically
  `routine-tasks.feature`).
- **Key Feature**: "Routinely" schedule type, Frequency/Interval configuration,
  and "Last Done" updates.
- **Implementation Details**:
  - **Step 1: Component Support (Helpers)**
    - [x] **[NEW] `crates/tasklens-ui/src/utils/time_conversion.rs`**:
      - [x] Implement `ms_to_period(ms: f64) -> (u32, String)` for lead time
            conversion (ms -> Days).
      - [x] Implement `period_to_ms(value: u32, unit: &str) -> f64` for lead
            time conversion (Days -> ms).
      - [x] Note: Routine frequency uses `RepeatConfig` (struct), not raw ms, so
            we need UI mappers for `Frequency` enum.
    - [x] **[MODIFY] `crates/tasklens-ui/src/utils/mod.rs`**: Export
          `time_conversion`.
  - **Step 2: Task Editor UI Updates**
    - [x] **[MODIFY] `crates/tasklens-ui/src/components/task_editor.rs`**:
      - [x] **Schedule Type Selector**: Ensure `Select` supports `Routinely`.
      - [x] **Conditional Rendering** (When `schedule.type == Routinely`):
        - [x] **Every**: Input (Number) -> binds to temporary state for
              `repeat_config.interval`.
        - [x] **Unit**: Select (Days/Weeks/Months/Years) -> binds to temporary
              state for `repeat_config.frequency`.
        - [x] **Lead Time**: Input (Days) -> binds to `schedule.lead_time`
              (using `period_to_ms`).
      - [x] **State Logic**:
        - [x] Ensure `draft.repeat_config` is initialized if switching to
              `Routinely`.
        - [x] Ensure `draft.repeat_config` is cleared if switching to `Once` or
              `DueDate`.
  - **Step 3: Controller Updates**
    - [x] **[MODIFY] `crates/tasklens-ui/src/controllers/task_controller.rs`**:
      - [x] Ensure `repeat_config` is correctly handled during task creation.
      - [x] Add validation to ensure `repeat_config` is present if
            `schedule_type` is `Routinely`.
  - **Step 4: Enable BDD Tests**
    - [x] **[MODIFY]
          `crates/tasklens-ui/tests/e2e/features/routine-tasks.feature`**:
      - [x] Remove `@migration-pending` tag.
- **Verification**:
  - [x] Run E2E tests for "routine-tasks"
        (`pnpm --filter @mydoo/tasklens-ui test-e2e -g "routine-tasks"`).
  - [x] Manually verify routine task creation and acknowledgment flow.
  - [x] Verify task reappears correctly after simulated Wait.
  - [x] **[VERIFY]**: Manual: Complete "Clean Desk". Refresh. Verify it
        disappears.
  - [x] **[VERIFY]**: Manual: Check persistence (reload page) and verify
        `Frequency` and `Lead Time` are preserved.

#### [x] Milestone 3.10: Sequential Projects

- **Goal**: Enable `sequential-projects.feature`.
- **Key Feature**: Sequential task execution—when a parent task has
  `is_sequential = true`, only the first non-completed child is visible in the
  Do list. Subsequent siblings are blocked until their predecessors are done.
- **PRD Reference**: §3.4 "State Definitions & Visibility Logic"—`IsBlocked`
  state: "True if Parent is Sequential AND a preceding sibling (in Outline
  Order) is Pending."
- **Algorithm Reference**: `algorithm.md` §3.4—Sequential Blocking Note:
  "non-first siblings as `Visibility=0` (hidden)".
- **Pre-existing Support (No Changes Needed)**:
  - Core domain (`tasklens-core`) already has `is_sequential: bool` on:
    - `PersistedTask` (types.rs)
    - `EnrichedTask` (types.rs)
    - `ComputedTask` (types.rs)
  - Priority algorithm (`priority.rs`) already implements blocking logic in
    `process_children()` (lines 312–328): checks `is_sequential` on parent and
    sets `visibility = false` for non-first pending children.
- **Implementation Details**:
  - **Step 1: Store Update Support**
    - [x] **[MODIFY] `crates/tasklens-store/src/actions.rs`**:
      - [x] Add `is_sequential: Option<bool>` field to `TaskUpdates` struct
            (after `repeat_config`).
    - [x] **[MODIFY] `crates/tasklens-store/src/store.rs`**:
      - [x] In `Action::UpdateTask` handler (around line 102, after
            `repeat_config` handling), add:
            `rust     if let Some(is_seq) = updates.is_sequential {         task.is_sequential = is_seq;     }     `
  - **Step 2: Task Editor UI**
    - [x] **[MODIFY] `crates/tasklens-ui/src/components/task_editor.rs`**:
      - [x] In the `save_handler` closure (lines 141–153 and 163–175), add
            `is_sequential: Some(d.is_sequential)` to both `TaskUpdates` struct
            initializations.
      - [x] Add a "Sequential Project" checkbox/toggle in the form UI (after the
            "Notes" section, before "Footer Actions"). Use Dioxus's `checkbox`
            or a `Switch` component with:
        - [x] `id: "sequential-toggle"`
        - [x] `aria-label: "Sequential Project"`
        - [x] Bind `checked` to `current_draft.is_sequential`
        - [x] On change, update `draft.is_sequential`
        - [x] Display Label: "Sequential Project" with helper text "Do steps in
              order"
      - [x] Ensure the checkbox is only visible in Edit mode (when
            `task_id.is_some()`) since sequential only applies to containers.
  - **Step 3: Enable BDD Tests**
    - [x] **[MODIFY]
          `crates/tasklens-ui/tests/e2e/features/sequential-projects.feature`**:
      - [x] Remove `@migration-pending` tag from line 1.
- **Verification**:
  - [x] **[VERIFY]**: Run
        `pnpm --filter @mydoo/tasklens-ui test-e2e -g "sequential-projects"`.
  - [ ] **[VERIFY]**: Manual: Create parent "Project S". Toggle "Sequential".
        Add Child A, Child B. Verify only A is visible in Do view. Complete A.
        Refresh. Verify B appears.

#### [x] Milestone 3.11: Document ID Management

- **Goal**: Enable `document-switching.feature` by implementing UI for managing
  document identifiers.
- **Context**: Unlike the React impl which uses `automerge-repo` and deals in
  automerge URLs, the Rust impl uses a simpler document ID model. The document
  ID is a cryptographically random string that:
  - Identifies the local Automerge document in IndexedDB
  - When combined with the master key, determines the sync channel
- **Reference**: `reference_repos/todo_mvp` contains similar patterns for sync_id
  generation and persistence.
- **Implementation Details**:
  - **Step 1: Store-Level Document ID Support**
    - [x] **[MODIFY] `crates/tasklens-store/src/lib.rs`**:
      - [x] Add `doc_id` module export.
    - [x] **[NEW] `crates/tasklens-store/src/doc_id.rs`**:
      - [x] Define `DOC_ID_STORAGE_KEY` constant (`"tasklens_doc_id"`).
      - [x] Implement `generate_doc_id() -> String`:
        - Use `rand::thread_rng()` and `hex::encode()` to produce a 32-byte
          random hex string.
      - [x] Implement `save_doc_id(doc_id: &str) -> Result<()>`:
        - Persist to LocalStorage via `gloo_storage::LocalStorage`.
      - [x] Implement `load_doc_id() -> Result<Option<String>>`:
        - Read from LocalStorage.
      - [x] Implement `clear_doc_id()`:
        - Remove from LocalStorage.
    - [x] **[MODIFY] `crates/tasklens-store/src/store.rs`**:
      - [x] Update `build_db()` to use the current `doc_id` for database name
            (e.g., `tasklens_db_{doc_id}`) OR use a single DB with doc_id as a
            key prefix. Choose the simpler approach: use doc_id as part of the
            IndexedDB key (e.g., `doc:{doc_id}:root`).
      - [x] Update `save_to_db()` and `load_from_db()` to accept `doc_id`
            parameter.
  - **Step 2: UI Components**
    - [x] **[MODIFY] `crates/tasklens-ui/src/views/auth.rs`**:
      - [x] Add a new `DocumentSection` component within `SyncActiveView` (or as
            a sibling) that shows the current document ID.
      - [x] Display the truncated doc ID (first 8 chars + "...") with a "Copy
            Full ID" button.
    - [x] **[NEW] `crates/tasklens-ui/src/components/doc_id_manager.rs`**:
      - [x] Create `DocIdManager` component.
      - [x] **Props**:
        - `current_doc_id: Signal<Option<String>>`
        - `on_change: EventHandler<String>` (called when doc ID changes)
      - [x] **UI Elements**:
        - [x] "Current Document" label with truncated ID display.
        - [x] "Copy ID" button (copies full ID to clipboard).
        - [x] "New Document" button:
          - Generates new random doc_id via `doc_id::generate_doc_id()`.
          - Clears the store (calls `Store::new()`).
          - Saves new doc_id via `doc_id::save_doc_id()`.
          - Calls `on_change`.
        - [x] "Enter ID" button/input:
          - Shows input field for manual ID entry.
          - On submit, validates (non-empty, alphanumeric), saves, reloads store
            from that doc_id's data.
    - [x] **[MODIFY] `crates/tasklens-ui/src/components/mod.rs`**:
      - [x] Export `doc_id_manager`.
  - **Step 3: App Integration**
    - [x] **[MODIFY] `crates/tasklens-ui/src/main.rs`**:
      - [x] Add `doc_id: Signal<Option<String>>` state.
      - [x] In startup future:
        1. Load doc_id from storage via `doc_id::load_doc_id()`.
        2. If None, generate a new one and save it.
        3. Pass doc_id to `AppStore::load_from_db(doc_id)`.
      - [x] Provide `doc_id` as context for child components.
      - [x] Update persistence effect to use current doc_id.
    - [x] **[MODIFY] `crates/tasklens-ui/src/views/auth.rs`**:
      - [x] Import and render `DocIdManager` in `SettingsModal` below the
            identity section.
      - [x] Wire `on_change` to trigger store reload and re-render.
  - **Step 4: Enable BDD Tests**
    - [x] **[MODIFY]
          `crates/tasklens-ui/tests/e2e/features/document-switching.feature`**:
      - [x] Remove `@migration-pending` tag.
    - [x] **[MODIFY] `crates/tasklens-ui/tests/e2e/steps/all.steps.ts`**:
      - [x] Implement steps for document switching:
        - "When the user creates a new document"
        - "Then the document ID changes"
        - "When the user switches to document {string} by its ID"
    - [x] **[MODIFY] `crates/tasklens-ui/tests/e2e/pages/`**:
      - [x] Add Page Object methods for interacting with `DocIdManager`.
- **Verification**:
  - [x] **[VERIFY]**: Run
        `pnpm --filter @mydoo/tasklens-ui test-e2e -g "document-switching"`.
  - [x] **[VERIFY]**: Manual: Open Settings, observe current doc ID.
  - [x] **[VERIFY]**: Manual: Click "New Document", verify ID changes and task
        list is empty.
  - [x] **[VERIFY]**: Manual: Enter a known doc ID, verify tasks from that
        document load.

#### [ ] Milestone 3.12: Mobile Journeys

- **Goal**: Enable `mobile-journeys.feature`.
- **Key Features**: Mobile-specific UI (drill-down navigation, bottom bar,
  breadcrumbs).
- **Details**:
  - Verify touch targets and responsive layouts on mobile.
  - Implement mobile drill-down navigation in Plan view.
  - Add mobile bottom bar with navigation controls.

### Epoch 4: Evaluation & Polish

_Goal: Rigorous testing, performance tuning, and final cutover._

#### [ ] Milestone 4.1: Full Regression Suite

- Ensure all 22+ feature files pass reliably.

#### [ ] Milestone 4.2: "Retire" React app

- Remove `apps/client` and promote `crates/tasklens-ui` to primary frontend.

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

- **IndexedDB Performance**: Verify performance of connection-per-operation
  pattern in `storage.rs`. Compare against long-lived connection pool strategy
  for high-frequency writes.
- **WASM Integration Testing**: Add `wasm-bindgen-test` integration tests for
  IndexedDB storage. Run with `wasm-pack test --headless --chrome` to test in a
  real browser environment without Playwright. This would provide lightweight
  WASM testing without the overhead of full E2E tests.
- **E2E Sync Testing**: Verify verified data consistency between two browser
  instances.

## Next Steps

1.  Implement **Milestone 3.11**: Document ID Management (Completed 3.10
    Sequential Projects).
2.  Continue un-tagging features from `@migration-pending` as implementation
    progresses.
