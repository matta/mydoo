# Code Review Guidance

This document captures best practices for code review in the mydoo repository.
It is intended for **human and AI reviewers** to ensure consistency,
maintainability, and type safety across the codebase.

## TypeScript Conventions

# TypeScript Strictness & Type Safety Rules

1.  **Zero-Tolerance for `any`:** Never use the `any` type. If a type is
    difficult to express, you must define an interface, use a generic, or
    utilize utility types (`Pick`, `Omit`, etc.) to construct it correctly.
2.  **Prohibition on Casting:** Do not use `as` casting (e.g.,
    `variable as Type`) or non-null assertions (`!`) to silence type errors.
    Casting is only acceptable when bridging boundaries (e.g., parsing raw JSON)
    and must be accompanied by runtime validation (like Zod) or a user-defined
    type guard.
3.  **Type Errors are Logical Defects:** Treat a compilation error as a
    structural defect in the code's logic or data flow, not a hurdle to be
    bypassed. If types do not match, change the implementation or the data
    structures to align—do not relax the type definition to satisfy the
    compiler.
4.  **No `unknown` Lazy-Loading:** Do not type variables as `unknown` to defer
    typing decisions. Only use `unknown` if the value is truly dynamic at
    runtime, and immediately narrow it using control flow analysis.
5.  **Exhaustiveness:** When handling unions (especially in `switch`
    statements), ensure all cases are handled. Use a `assertUnreachable` utility
    if necessary to guarantee exhaustiveness.
6.  **The Protocol of Strictness:** If you cannot express a type correctly
    without using `any` or `as` casting, **STOP**. A complex type puzzle you
    cannot solve is a signal to pause and ask the user for guidance, not a
    license to bypass the type system. Lowering the quality bar to achieve
    autonomy is **unacceptable**.

### Prefer `undefined` over `null`

Use `undefined` to represent the absence of a value. Reserve `null` only when
interfacing with APIs that idiomatically use it (e.g., React
`useState<Date | null>(null)` for date pickers).

**Rationale**: `undefined` is TypeScript's natural sentinel for "no value" and
simplifies optional chaining. Mixing `null` and `undefined` leads to verbose
guards like `value !== null && value !== undefined`.

```typescript
// ✅ Good
function getTask(id: TaskID): Task | undefined {
  return tasks[id];
}

// ❌ Avoid
function getTask(id: TaskID): Task | null {
  return tasks[id] || null;
}
```

---

## React Hooks

### Keep `useMemo` and `useCallback` Bodies Concise

The total line count for the arguments to a `useMemo` or `useCallback` call
should be **less than 5 lines**. Factor complex logic into named helper
functions.

**Rationale**: Concise hook bodies improve readability and make dependency
arrays easier to audit.

```typescript
// ✅ Good
const details = useMemo(
  () => projectTaskDetails(doc, task, taskId),
  [doc, task, taskId],
);

// ❌ Avoid
const details = useMemo(() => {
  if (!task || !doc) {
    return { task: undefined, parentTitle: undefined, descendantCount: 0 };
  }
  const parentTask = task.parentId ? doc.tasks[task.parentId] : undefined;
  const parentTitle = parentTask?.title;
  const descendantCount = TunnelOps.getDescendantCount(doc, taskId);
  return { task, parentTitle, descendantCount };
}, [doc, task, taskId]);
```

---

## Redux

### Always Name Selectors

Redux selectors passed to `useSelector` **must be named functions**. Anonymous
inline selectors make debugging difficult and trigger "Selector unknown"
warnings in React DevTools.

**Rationale**: Named selectors provide clear stack traces and enable better
diagnostics.

```typescript
// ✅ Good
export function selectLastDoc(state: RootState) {
  return state.tasks.lastDoc;
}

const doc = useSelector(selectLastDoc);

// ❌ Avoid
const doc = useSelector((state) => state.tasks.lastDoc);
```

For parameterized selectors, use factory functions that return named inner
functions:

```typescript
// ✅ Good
export function selectTaskById(id: TaskID | undefined) {
  return function selectTask(state: RootState): ComputedTask | undefined {
    if (!id) return undefined;
    return state.tasks.entities[id];
  };
}
```

---

## Testing

### Fidelity First

Prefer **Browser Mode** over JSDOM for component tests. Browser Mode provides
higher fidelity and catches issues that JSDOM misses.

### Use `toBeUndefined()` Not `toBeNull()`

Align test assertions with the "prefer `undefined`" convention:

```typescript
// ✅ Good
expect(result.current.task).toBeUndefined();

// ❌ Avoid
expect(result.current.task).toBeNull();
```

---

## Linting

### Suppression is a Last Resort

Suppressing lint errors is a last resort -- there is almost always a better way.

**Rationale**: Bypassing checks strictly reduces the safety and maintainability
of the codebase.

### Avoid Casting through `unknown`

Casting through `unknown` (e.g., `foo as unknown as Bar`) is banned. This
pattern disables the compiler and hides contract violations that surface as
cryptic runtime errors.

For test mocks specifically, see
[Type-Safe Mocking Strategies](type-safe-mocking.md) for approved alternatives
including interface segregation and the `strictMock` utility.

---

## Rust & Dioxus Conventions

### Prefer Top-Level Imports

Place all `use` statements at the top of the file. Avoid nesting `use`
statements inside function bodies or `rsx!` blocks.

**Rationale**: Nested imports make dependencies harder to track and lead to
redundant imports in large component files.

### Avoid Silent Failures in Event Handlers

Never use silent failures (e.g., `if let Ok(...) { ... }` without an `else`) in
UI event handlers. Use `tracing::warn!` to log parsing or logic errors.

**Rationale**: Debugging WASM in the browser is difficult. Without explicit
logging, it is impossible to distinguish between a intentional "no-op" and an
unexpected failure.

---

## General

### PR Scope Hygiene

Before creating or updating a PR, confirm branch and PR scope:

1. Each semantically independent task should have its own feature branch.
2. Each PR should contain one cohesive concern.
3. If the branch already has an open PR, only add commits that are in scope for
   that PR.
4. If the new work is independent, create a new branch and a new PR instead of
   reusing the existing one.

### Verification Before Commit

Before submitting a PR, run:

```bash
just verify
```

This is the **full verification gate** that runs auto-fixes (`just fix`)
followed by all checks, unit tests, and E2E tests.

> [!IMPORTANT] Use `just verify`, not `just check`, as your final verification
> step.
>
> - `just check` runs all static analysis (linting, type checking, rust checks).
> - `just verify` runs `just fix` (auto-formatting), `just check` (analysis),
>   `just test` (unit tests), and `just test-e2e` (E2E tests).
