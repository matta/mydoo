# Code Review Guidance

This document captures best practices for code review in the mydoo repository.
It is intended for **human and AI reviewers** to ensure consistency,
maintainability, and type safety across the codebase.

## TypeScript Conventions

# TypeScript Strictness & Type Safety Rules

1.  **Zero-Tolerance for `any`:** Never use the `any` type. If a type is difficult to express, you must define an interface, use a generic, or utilize utility types (`Pick`, `Omit`, etc.) to construct it correctly.
2.  **Prohibition on Casting:** Do not use `as` casting (e.g., `variable as Type`) or non-null assertions (`!`) to silence type errors. Casting is only acceptable when bridging boundaries (e.g., parsing raw JSON) and must be accompanied by runtime validation (like Zod) or a user-defined type guard.
3.  **Type Errors are Logical Defects:** Treat a compilation error as a structural defect in the code's logic or data flow, not a hurdle to be bypassed. If types do not match, change the implementation or the data structures to align—do not relax the type definition to satisfy the compiler.
4.  **No `unknown` Lazy-Loading:** Do not type variables as `unknown` to defer typing decisions. Only use `unknown` if the value is truly dynamic at runtime, and immediately narrow it using control flow analysis.
5.  **Exhaustiveness:** When handling unions (especially in `switch` statements), ensure all cases are handled. Use a `assertUnreachable` utility if necessary to guarantee exhaustiveness.

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

## General

### Verification Before Commit

**Always** run the following before committing:

```bash
pnpm verify
```

This is the **full verification gate** that runs auto-fixes (`pnpm fix`)
followed by all checks (`pnpm check-agent`).

> [!IMPORTANT] Use `pnpm verify`, not `pnpm check`, as your final verification
> step.
>
> - `pnpm check` runs linting, type checking, and tests
> - `pnpm verify` runs `pnpm fix` first (auto-formatting), then
>   `pnpm check-agent` (all checks)
>
> The `verify` command matches the pre-commit hook and CI pipeline behavior.
