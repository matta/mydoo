# Pattern: Exhaustive Destructuring

## Goal

To enforce **Exhaustiveness Checking** on strict TypeScript data structures
without the runtime cost of loops like `Object.keys()`. This ensures that when a
property is added to a Type, the compiler forces all consumers to handle it.

## Applicability

Best for **Fixed-Shape Data Types** (Product Types) with known keys.

- ✅ **Interfaces / Type Aliases** (`interface User { ... }`)
- ✅ **Data-Only Classes** (DTOs)
- ⛔ **Classes with Methods** (Methods require awkward destructuring)
- ⛔ **Index Signatures** (Open-ended types)

## When to Use

1.  **Product Type Consumers:** For functions that must process every field of
    an object (mappers, serializers).
2.  **No Discriminator:** Use this when you can't use `switch` because the type
    lacks a `kind` field.
3.  **Schema Safety:** To ensure schema changes break the build, forcing manual
    review of all call sites.

> **Note:** For Sum Types (Unions), prefer `switch` with `default: never`.

## The Pattern

```typescript
type User = {
  id: number;
  username: string;
  isActive: boolean;
  // Adding "email: string;" triggers a build error below.
};

function processUser(user: User): void {
  // 1. DESTRUCTURE: Extract known fields. Collect the rest.
  const { id, username, isActive, ...rest } = user;

  // 2. BAN: Assert 'rest' is empty (Record<string, never>).
  //    Fails if 'rest' contains any keys (including optional ones).
  const _exhaustiveCheck: Record<string, never> = rest;

  // 3. SILENCE: Mark check as used for 'noUnusedLocals'.
  //    This statement is erased at compile time.
  void _exhaustiveCheck;

  // 4. LOGIC: Use extracted variables.
  console.log(id, username, isActive);
}
```

## How It Works

1.  **Completeness:** `...rest` captures any property not explicitly
    destructured.
2.  **Assertion:** Assigning `rest` to `Record<string, never>` fails if `rest`
    has any properties.
3.  **Safety:** `noUnusedLocals` forces usage of the extracted variables.

## Performance

- **Runtime Cost:** **Low.** Destructuring allocates a shallow object for
  `rest`.
- **Guidance:** Safe for business logic. Avoid in hot loops (e.g., per-frame
  rendering). Significantly faster than `Object.keys()` iteration.
