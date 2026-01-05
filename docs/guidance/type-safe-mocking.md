# Engineering Guidance: Type-Safe Mocking Strategies

**Status:** Active
**Context:** Strict TypeScript Repository

## The Problem

We have observed a pattern of "unsafe mocking" in tests using double-casting hacks:

```typescript
const mock = { ... } as unknown as ServiceType;
```

> [!CAUTION]
> **This pattern is now banned.**
>
> - It disables TypeScript's contract checking.
> - If `ServiceType` changes (e.g., a method is renamed), the test compiles but fails at runtime with cryptic errors like `undefined is not a function`.

## The Solution Hierarchy

We prioritize testability at the architectural level. Follow this decision tree:

1. **Level 1 (Preferred):** Narrow the Interface (Interface Segregation)
2. **Level 2 (Tooling):** Use `strictMock` (The Strict Proxy Pattern)
3. **Level 3 (Legacy/Large):** Use `jest-mock-extended` (The Permissive Proxy Pattern)

---

## Level 1: Interface Segregation (The Architectural Fix)

**"Don't mock what you don't use."**

The best way to mock a large object is to define that your function only needs a small slice of it. This relies on TypeScript's structural typing (duck typing).

**Instead of depending on a concrete class:**

```typescript
// BAD: Tightly coupled to the entire heavy database service
class UserService {
  constructor(private db: PostgresDatabaseService) {}
}
```

**Depend on a narrow interface:**

```typescript
// GOOD: Defines exactly what is needed
interface UserSaver {
  save(user: User): Promise<void>;
}

class UserService {
  constructor(private db: UserSaver) {}
}
```

**Testing becomes trivial:**

```typescript
// The compiler guarantees this object is valid for the narrowed interface
const mockDb = {save: vi.fn()};
new UserService(mockDb);
```

---

## Level 2: `strictMock` (The Strict Proxy Pattern)

**"Fail fast if I touch something I didn't mock."**

If you cannot refactor the interface (e.g., testing a legacy integration), use the `strictMock` utility. This creates a Proxy that satisfies the full TypeScript type but **throws a runtime error** if the code under test accesses a property you did not explicitly mock.

**Use when:**

- You want to verify that the code _only_ uses specific methods.
- You want tests to explode loudly if they touch unexpected dependencies.

**Location:** [`packages/tasklens/src/test/test-utils.ts`](file:///Users/matt/src/mydoo/packages/tasklens/src/test/test-utils.ts)

**Usage:**

```typescript
import {strictMock} from '@mydoo/tasklens/test';

it('processes the user', () => {
  const db = strictMock<Database>('Database', {
    getUser: vi.fn().mockReturnValue({id: 1}),
    // If the code calls db.deleteUser(), the test fails instantly.
  });

  processUser(db);
});
```

---

## Level 3: `jest-mock-extended` (The Permissive Proxy Pattern)

**"I need a mostly-empty object that just works."**

Use this sparingly. It is useful for massive DTOs or context objects (like an Express `Request` or `Response`) where the code might touch many properties safely.

**Use when:**

- Refactoring to narrower interfaces is impossible.
- The object is a data container rather than a behavior service.

**Usage:**

```typescript
import {mock} from 'jest-mock-extended';

// Creates a proxy where EVERY property returns a default spy/mock.
const req = mock<Request>();
req.body = {data: 'test'};

handler(req);
```

---

## Summary Cheat Sheet

| Scenario                                                  | Recommended Approach               | Why?                                                                                |
| --------------------------------------------------------- | ---------------------------------- | ----------------------------------------------------------------------------------- |
| New feature development                                   | **Level 1 (Narrow Interface)**     | Zero mocking overhead; cleaner architecture.                                        |
| Testing logic that relies on a specific dependency method | **Level 2 (`strictMock`)**         | Guarantees the test fails if the code starts using other dependencies unexpectedly. |
| Mocking huge data objects (e.g., `window`, `req`)         | **Level 3 (`jest-mock-extended`)** | Reduces boilerplate setup for noisy objects.                                        |
| `as unknown as Type`                                      | **BANNED**                         | Unsafe. Hides errors.                                                               |
