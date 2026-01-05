# Coding Conventions

## Linting & Static Analysis

### Suppression Policy

We adhere to a strict policy regarding linting suppressions to ensure code
quality and prevent technical debt accumulation.

#### General Principles

1.  **Suppress As A Last Resort**: Suppressions should be used sparingly and
    only when absolutely required. Avoid suppressing entire files or large
    sections of code. There is usually a better way to address the underlying
    issue; strongly consider refactoring the code to remove the need for the
    suppression.
2.  **No Blanket Suppressions**: File-level suppressions are **strictly
    prohibited**.
3.  **Targeted Suppressions Only**: Suppressions must be as granular as
    possible, using `// biome-ignore lint/rule: reason`.
4.  **Mandatory Justification**: Every suppression **must** be accompanied by a
    explanation of _why_ the suppression is necessary (which is built into the
    biome-ignore syntax).

**Correct:**

```typescript
// Safe because we are testing the error handler
// biome-ignore lint/suspicious/noExplicitAny: testing error handler
const invalidInput = 'foo' as any;
```

**Incorrect:**

```typescript
// biome-ignore lint/suspicious/noExplicitAny: reason
const invalidInput = 'foo' as any;
```

### Type Safety

- **Avoid `any`**: Use `unknown` or specific types whenever possible. If `any`
  is absolutely required (e.g., for testing boundary conditions or working with
  untyped libraries), it must be suppressed according to the policy above.
