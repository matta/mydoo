---
id: issue-xqyyw005kvu
title: "Phase 1: Migrate TaskEditor to Full-Page Route Navigation"
status: todo
priority: 9
created_at: 2026-03-04T04:02:57.199260682+00:00
modified_at: 2026-03-04T04:45:05.252750546+00:00
tags:
  - task
  - ui
  - css
---

Migrate `TaskEditor` to use Full-Page Route Navigation.

Rationale:
Implementing a List-Detail (Two-Pane) layout carries implementation complexity regarding Dioxus routing vs screen resolution. The most prudent and robust way to build it is to start with pure Full-Page Routing (Phase 1) and layer on List-Detail (Two-Pane) layouts later (Phase 2). This guarantees excellent mobile UX from day one.

Scope:

- Add canonical editor routes as the single source of truth across all views (Plan, Do, Search, Find-in-Plan):
  - `/task/new?ctx=plan` (or `ctx=do`) `&parent_id=[TaskID]`
  - `/task/:id/edit?[ViewContext Params]`
  - We use a flattened, strictly-typed Sum Type (`ViewContext`) to define the allowed parameters, ensuring invalid combinations (like `highlight` on a `search` entry) are mathematically unrepresentable.
  - The `ViewContext` enum dictates the route shape:
    | Context Variant | Query String | Exit Target |
    | :--- | :--- | :--- |
    | **Plan** | `?ctx=plan` | `/plan` |
    | **Do** | `?ctx=do` | `/do` |
    | **Search** | `?ctx=search&return_to=[/plan\|/do]` | `/plan` or `/do` |
    | **Find-in-Plan** | `?ctx=find_in_plan&highlight=[TaskID]` | `/plan` |
  - In-app navigation always emits valid `ViewContext` query combinations. Direct deep-link URL entry with missing/invalid fields is unsupported user error; behavior is intentionally unspecified and untested.
- `ViewContext` Invariant: `ViewContext` variants are the only valid states. Component-level invalidity concepts (such as a `valid return_to` coupled with an invalid sibling field) are completely out-of-model and must not appear in requirements text. Thanks to atomicity and serialization bounds, invalid states are unrepresentable.
- Router Type Safety: Implement `ViewContext` using strictly-typed Rust Enums `#[serde(tag = "ctx", rename_all = "snake_case")]` and flatten it into the router query struct. This forces `serde` to handle the parsing logic, automatically rejecting bad variants and collapsing the need for complex fallback rules.

  ```rust
  #[derive(Clone, PartialEq, Serialize, Deserialize)]
  pub struct EditorQuery {
      #[serde(flatten)]
      pub context: ViewContext,
  }

  #[derive(Clone, PartialEq, Serialize, Deserialize)]
  #[serde(tag = "ctx", rename_all = "snake_case")]
  pub enum ViewContext {
      Plan,
      Do,
      Search { return_to: ReturnRoute },
      FindInPlan { highlight: TaskId },
  }
  ```

- Both `PlanPage` and `DoPage` no longer import or render `<TaskEditor />` directly. Clicking a task (`on_title_tap` / create) pushes the canonical enum route to the navigator.
- Exact Return Semantics:
  - **Dioxus Navigation API Rule**: Tasklens UI code must use Dioxus Navigation APIs exclusively. Close actions (UI controls and programmatic exits) use `navigator().go_back()`.
  - **Deep-Link Non-Goal**: Direct deep-link arrivals to editor routes are unsupported and untested. We do not provide fallback, normalization, or recovery requirements for those entries.
- Search Context Policy:
  - Search remains temporary UI state, not a URL-driven route state.
  - `ctx=search` records provenance only and does not define return destination (it uses its internal `return_to` field).
  - Search query text is dropped intentionally after result selection and is not restored on Back from editor.
  - Back from editor returns to the `return_to` underlying context, not the search view.
- Find-in-Plan Context Policy:
  - Find-in-Plan launches into editor uses `ctx=find_in_plan` and tracking parameter `highlight=[TaskID]`.
  - The concrete state key `highlight=[TaskID]` alone drives all restoration and must be included to signal restoration intent on Back.
  - Returning to `/plan` with `highlight=[TaskID]` directs the Plan view to safely re-expand the collapsed tree up to the task, scroll the task into the viewport offset, and apply visual highlight targeting that row. Find query text is intentionally not restored.
- Edge Case Fallbacks: Define behavior for missing/deleted task IDs in route, and concurrent mutations while route is open.
- UX Policies: Implement immediate write commit edge policies instead of draft/save. All task editor fields persist immediately on blur, explicit toggle, or navigation unmount. There are no UI "Save" or "Cancel" buttons, as the back button serves to finish editing. Define loading/error states for route-driven pages.
- Persistence Safety: Focused-but-unflushed input is safely committed during browser Back, programmatic navigation out, and breakpoint/orientation layout transitions.
- Speed Path Preservation: Keep the inline title-only quick-add capture bar on list views. It should NOT force full-page navigation. Quick-add escalates to the full editor route only if the user explicitly chooses to add extra details to the draft.

Acceptance:

- Perfect mobile experience. The browser's back button works natively to navigate.
- Multi-Entry Parity: The new route-based editing fully supports and behaves consistently when initiated from Plan, Do, Search, and Find-in-Plan.
- Return-Target Preservation: Returning from the canonical editor route is determined strictly by `ViewContext` variant semantics (`Plan`, `Do`, `Search{return_to}`, `FindInPlan{highlight}`). When returning to `/plan` via `FindInPlan{highlight}`, the app must correctly restore the UI state (expanded/collapsed tree, scroll offset, highlighted row).
- Search Entry/Exit: Selecting a task from search opens the editor with `ctx=search`; pressing Back returns to the underlying context safely, proving deterministic navigation without restoring dropped search text.
- Enum Discipline: Plan/Do launches use `ctx=plan`/`ctx=do`; Search launches include `ctx=search` and `return_to`; Find-in-Plan launches include `ctx=find_in_plan` and `highlight`.
- Deep-link Non-Goal: Direct deep-link editor entries (valid or malformed) are unsupported user error and excluded from acceptance testing.
