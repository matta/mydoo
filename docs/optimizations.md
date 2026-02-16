# Optimization Opportunities

This document tracks potential performance optimizations identified during development. These should be addressed in a principled way with benchmark-based validation when performance becomes a critical focus.

## Frontend Optimizations

### 1. Reduce String Allocations in Task Lists
**Location:** `crates/tasklens-ui/src/views/plan_page.rs` and `crates/tasklens-ui/src/app_components/task_row.rs`

**Problem:**
The `TaskID` type is a wrapper around `String`. In the current implementation of `FlattenedTask` (view model) and `TaskRow` (component), `TaskID` and `title` (String) are cloned repeatedly:
1. When creating `FlattenedTask` instances in the `use_memo` hook.
2. When passing these values as props to `TaskRow`.
3. Inside `TaskRow`, they are cloned ~5 times for various event handler closures.

For a list of N tasks, this results in O(N) string allocations per render cycle. This is particularly noticeable during rapid updates (e.g., typing in the task input) or when the list grows large.

**Proposed Solution:**
Wrap heavy string-based fields in `std::rc::Rc` within the transient `FlattenedTask` struct and update `TaskRow` props to accept `Rc<TaskID>` and `Rc<String>`.

- **Change:** `id: TaskID` -> `id: Rc<TaskID>`
- **Change:** `title: String` -> `title: Rc<String>`

This replaces expensive heap allocations with cheap pointer copies (reference counting increases).

**Benchmarks needed:**
- Measure render time for lists of 100, 500, and 1000 tasks.
- Measure memory usage during rapid input typing.
