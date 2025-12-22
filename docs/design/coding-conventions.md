# Coding Conventions

## Linting & Static Analysis

### Suppression Policy

We adhere to a strict policy regarding linting suppressions to ensure code quality and prevent technical debt accumulation.

#### General Principles

1.  **Suppress As A Last Resort**: Suppressions should be used sparingly and only when absolutely required. Avoid suppressing entire files or large sections of code. There is usually a better way to address the underlying issue; strongly consider refactoring the code to remove the need for the suppression.
2.  **No Blanket Suppressions**: File-level suppressions (e.g., `/* eslint-disable ... */` at the top of a file) are **strictly prohibited**.
3.  **Targeted Suppressions Only**: Suppressions must be as granular as possible, typically using `// eslint-disable-next-line`.
4.  **Mandatory Justification**: Every suppression **must** be accompanied by a comment on the preceding line explaining _why_ the suppression is necessary.

**Correct:**

```typescript
// Safe because we are testing the error handler
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const invalidInput = 'foo' as any;
```

**Incorrect:**

```typescript
/* eslint-disable @typescript-eslint/no-explicit-any */
const invalidInput = 'foo' as any;
```

### Type Safety

- **Avoid `any`**: Use `unknown` or specific types whenever possible. If `any` is absolutely required (e.g., for testing boundary conditions or working with untyped libraries), it must be suppressed according to the policy above.
