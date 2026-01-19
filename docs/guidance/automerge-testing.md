# Automerge Testing Guidance

This guide outlines idioms and patterns for testing Automerge document
operations and state, derived from the core Automerge codebase.

## Core Philosophy

Tests should be **concise**, **declarative**, and **stable**. We prioritize
readability of the _expected state_ over the imperative steps to get there, and
we use tools to ensure deterministic conflict resolution.

## Concise Expectations (`assert_doc!`)

The primary idiom for verifying document state is the `assert_doc!` macro (and
its companion `assert_obj!`). This allows you to declare the expected JSON-like
structure of the document including conflict sets.

### The `map!` and `list!` Macros

Instead of manually traversing the document or constructing complex `serde_json`
objects, use the `map!` and `list!` macros to define the expected structure.

**Key idioms:**

- **Structure First:** Define the shape of the data.
- **Conflict Sets:** Every leaf value in `assert_doc!` is implicitly a _set_ of
  values to account for conflicts. Even if you expect a single value, it is
  wrapped in `{ ... }` within the macro.

```rust
use automerge_test::{assert_doc, map, list, new_doc};
use automerge::{transaction::Transactable, ROOT};

#[test]
fn example_test() {
    let mut doc = new_doc();
    let todos = doc.put_object(ROOT, "todos", automerge::ObjType::List).unwrap();
    let todo = doc.insert_object(todos, 0, automerge::ObjType::Map).unwrap();
    doc.put(todo, "title", "water plants").unwrap();

    // Verify the entire document structure
    assert_doc!(
        &doc,
        map! {
            "todos" => {
                list![
                    {
                        map! {
                            "title" => { "water plants" }
                        }
                    }
                ]
            }
        }
    );
}
```

### Handling Conflicts

When testing concurrent operations, you can explicitly state multiple values for
a single field.

```rust
assert_doc!(
    &doc,
    map! {
        "field" => {
            "value_from_actor_A",
            "value_from_actor_B"
        }
    }
);
```

## Concise Mutations (`AutoCommit`)

For most unit tests, avoid managing `Transaction` lifecycles manually. Use
`automerge::AutoCommit`.

- **Pattern:** `let mut doc = AutoCommit::new();` (or `new_doc()`)
- **Benefit:** Operations apply immediately. No need to call `.commit()`.
- **Usage:** Standard `put`, `insert`, `delete` methods return `Result`. In
  tests, it is idiomatic to `.unwrap()` these results to fail fast if the
  operation is invalid.

```rust
doc.put(ROOT, "key", "value").unwrap();
```

## Stable Tests & Determinism

Automerge relies on Actor IDs to break ties in concurrent operations. Random
Actor IDs can lead to flaky tests where "Winner A" wins in one run and "Winner
B" in another.

### Controlling Actor IDs

To ensure stable sorting of concurrent changes:

1.  **`sorted_actors()`**: Returns two actors `(a, b)` such that `a < b`. Use
    this when you need predictable conflict resolution order.
2.  **`new_doc_with_actor(actor_id)`**: Initialize documents with specific
    actors.

```rust
use automerge_test::{sorted_actors, new_doc_with_actor};

#[test]
fn stable_conflict_resolution() {
    let (actor_a, actor_b) = sorted_actors();
    let mut doc_a = new_doc_with_actor(actor_a);
    let mut doc_b = new_doc_with_actor(actor_b);

    // ... make concurrent changes ...
    // actor_a's changes will predictably order vs actor_b's based on the sort order
}
```

## Debugging

If assertions fail, the `assert_doc!` macro provides a diff-like output using
`pretty_panic`. For ad-hoc debugging, use `pretty_print(&doc)` to dump the
current realized state of the document to stdout.

```rust
use automerge_test::pretty_print;

// Dumps the full document state with conflicts to stdout
pretty_print(&doc);
```
