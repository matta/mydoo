# Code Review Guidance

This document captures best practices for code review in the mydoo repository.
It is intended for **human and AI reviewers** to ensure consistency,
maintainability, and type safety across the codebase.

## TypeScript Conventions

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
    return {task: undefined, parentTitle: undefined, descendantCount: 0};
  }
  const parentTask = task.parentId ? doc.tasks[task.parentId] : undefined;
  const parentTitle = parentTask?.title;
  const descendantCount = TunnelOps.getDescendantCount(doc, taskId);
  return {task, parentTitle, descendantCount};
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
const doc = useSelector(state => state.tasks.lastDoc);
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

This ensures linting, formatting, and all tests pass.
