# Binary Crate `pub` Visibility Plan (Radical Simplicity)

## Summary

Implement a small xtask that catches most accidental bare `pub` in workspace binary crates using regex-based source scanning and explicit comment suppressions.

Important boundary:

- `unreachable_pub` is enforced by normal `rustc`/`clippy` workflows.
- This xtask does not run, replace, or suppress compiler `unreachable_pub` diagnostics.
- This xtask enforces a separate project policy.

## Goals

- Catch most accidentally public APIs in binary crates.
- Keep implementation radically simple and easy to maintain.
- Auto-discover binary crates from workspace metadata (no hardcoded crate list).
- Report violations with `miette`.
- Keep false positives low, especially for comments/doc comments.
- Keep suppression mechanics intentionally minimal for v1 (suppressions are expected to be rare).

## Non-Goals

- Perfect semantic accuracy.
- Library crate enforcement in this phase.
- Full AST/module-graph analysis.
- Reusing `#[allow(unreachable_pub)]` / `#[expect(unreachable_pub)]` for xtask suppression.
- Handling every parser edge case (for example complex multiline constructs).

## Why Not Lint-Only

`unreachable_pub` remains useful and should continue to be enforced by compiler/lint commands, but it is not identical to this policy check.

This plan intentionally adds a separate, simpler guardrail for accidental source-level bare `pub`.

## Scope

In scope:

- binary target discovery
- regex-based scan of Rust source files in binary crates
- comment-based suppression for xtask diagnostics
- `miette` output and CI integration

Out of scope:

- proving full reachability correctness
- macro expansion analysis
- auto-fix rewrites

## Detailed Design

### 1. Binary Target Discovery

Use `cargo_metadata` (same underlying metadata source Cargo uses) and discover all workspace targets with `target.kind` containing `"bin"`.

This ensures newly added binary crates are automatically covered.

### 2. File Selection

For each discovered binary crate, glob all `src/**/*.rs` files.

Rationale:

- maximally simple
- easy to reason about
- acceptable with relaxed fidelity goals

### 3. Detection Heuristic (Regex)

Scan files line-by-line and flag bare `pub` declarations with a regex equivalent to:

`^\s*pub\s+(fn|struct|enum|trait|type|const|static|mod|use)\b`

Additional rule:

- do not match `pub(...)` forms (`pub(crate)`, `pub(super)`, `pub(in ...)`)

Comment handling:

- skip `//`, `///`, and `//!` lines
- skip text inside `/* ... */` block comments via a simple block-comment state flag

This keeps the implementation simple while preventing common comment/doc-comment false positives.

Deliberate v1 limitation:

- no special handling for multiline string literal edge cases
- acceptable because this pattern is not present in the current codebase
- no full support for nested block-comment parsing
- no full support for newline-separated declarations like `pub` on one line and `fn` on the next

### 4. Suppression Model (Independent of `unreachable_pub`)

Suppressions are specific to this xtask and do not use `allow/expect(unreachable_pub)`.

Allowed suppression forms:

- Same-line:
  `pub fn foo() {} // xtask-pub-visibility: allow <reason>`
- Previous-line:
  `// xtask-pub-visibility: allow <reason>`
  associated with the next `pub ...` line across an optional contiguous attribute/doc block

Canonical attributed-item example:

```rust
#[derive(Debug, Clone)]
// xtask-pub-visibility: allow integration testing
pub struct AppState {
    // ...
}
```

Ergonomics note:

- v1 does not optimize for every placement style
- this is acceptable because suppressions should be uncommon

Association rule (v1, low-complexity):

- when a `pub ...` match is found, scan upward through a contiguous block of:
  - attribute lines (`#[...]`)
  - doc comment lines (`///`, `//!`)
  - blank lines
- if a suppression comment is found before any other code line, treat it as applying to that `pub`
- otherwise, require same-line suppression

If suppression text is malformed or reason is missing, xtask still reports a violation.

Optional path exclusions:

- maintain a minimal exclusion list for known special directories (for example vendored paths).

### 5. Diagnostics (`miette`)

Each violation is emitted as a `miette` diagnostic containing:

- file path and line/column
- highlighted span for `pub`
- crate/target context
- suggested fix (`pub(crate)` or narrower)

Behavior:

- emit all violations in a single run
- exit non-zero if any violations exist

## Integration

- Add command: `cargo xtask check-bin-pub-visibility`
- Register in `xtask` and include in `CheckAll`
- Continue relying on existing compiler/lint commands for `unreachable_pub`

## Trade-offs and Known Limitations

- Regex scanning can miss complex syntax and macro-generated items.
- Globbing all `src/**/*.rs` may include files not active in a specific cfg/module path.
- Some false positives/negatives are possible by design.
- Suppression placement rules are intentionally strict/minimal in v1.
- Multiline string literal corner cases are intentionally unsupported in v1.
- Nested block-comment corner cases are intentionally unsupported in v1.
- Line-break evasion (`pub` newline `fn`) is intentionally unsupported in v1.

These are accepted trade-offs for radical simplicity and low implementation cost.

## Future Direction: `syn` Upgrade Path

If regex shortcomings become operationally expensive, migrate the scanner to `syn` parsing while preserving the same command shape and suppression policy intent.

Expected benefits of a future `syn` version:

- better item-level accuracy
- cleaner attribute/structure handling
- fewer heuristic edge cases than regex scanning
- robust handling of currently accepted v1 limitations (for example suppression placement edge cases and multiline constructs)
- robust handling of nested block comments and newline-split declarations without ad-hoc regex state logic

## Rollout Plan

### Phase 1: Report-Only

- implement command
- run as non-blocking in CI

### Phase 2: Cleanup

- fix current findings
- add minimal justified suppressions/exclusions

### Phase 3: Enforce

- fail CI on violations via `check-all`

## Success Criteria

- all workspace binary crates are auto-discovered
- most accidental bare `pub` additions in binary crates are caught
- diagnostics are consistently reported through `miette`
- suppressions are explicit and local to this xtask policy
