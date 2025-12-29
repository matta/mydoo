# Derived State: Selectors vs Hooks

This document provides guidance on when to use Redux Selectors versus React Hooks with `useMemo` for computing derived state.

## Decision Criteria

### Prefer Selectors when:
1.  The derived data is used by **more than one component** (or might be in the future).
2.  The data represents a **domain concept** (e.g., "Balance Allocation," "Top Priority Tasks"), not a UI-specific transform.
3.  The calculation is **expensive** and benefits from global memoization.
4.  You need to **compose** this derived data into further derivations.
5.  You need the data outside React (e.g., in middleware, for logging, or for server sync logic).

### Prefer Hooks with `useMemo` when:
1.  The logic is **tightly coupled to local UI state** (e.g., a search filter, a sort toggle).
2.  The transformation is **trivial** (e.g., `tasks.filter(t => t.isReady)`).
3.  The data is only ever needed by **a single component instance**.
4.  You're prototyping and want to move fast before deciding on the "right place."

## Comparison Table

| Criterion | Hook + `useMemo` | Redux Selector |
|---|---|---|
| Memoization Scope | Per-component instance | Global singleton |
| Cache Invalidation | Lost on unmount | Persistent for app lifetime |
| Composition | Awkward, nested hooks | First-class with `createSelector` |
| Testability | Requires `renderHook` | Pure function, no React needed |
| DevTools Visibility | Invisible | Visible in Redux DevTools |
| Dependency on React | Yes | No |
| Local UI State Integration | Natural | Requires parameterized selectors |

## Common Patterns

### Domain-level derived state (use Selector)
```typescript
// In store/selectors.ts
export const selectBalanceData = createSelector(
  [selectTaskEntities],
  entities => calculateBalanceData(Object.values(entities))
);

// In component
const balanceData = useSelector(selectBalanceData);
```

### UI-specific derived state (use Hook)
```typescript
// In component
const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');
const tasks = useTaskList();

const sortedTasks = useMemo(
  () => [...tasks].sort((a, b) => 
    sortOrder === 'asc' ? a.priority - b.priority : b.priority - a.priority
  ),
  [tasks, sortOrder]
);
```

## Migration Path

If you start with a hook and later realize it should be a selector:
1.  Extract the pure logic into a domain function (e.g., `domain/balance.ts`).
2.  Create a selector that calls the domain function.
3.  Simplify the hook to just call `useSelector(selectX)`.

This "pure function first" approach keeps logic testable regardless of where it's called from.
