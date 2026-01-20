# Autosurgeon Guidance

This guide explains how to use `autosurgeon` to perform "surgical" mutations on
Automerge documents. While Automerge provides low-level APIs for manipulating
the document structure, `autosurgeon` allows you to work with Rust structs and
leverage the type system to perform precise updates.

## Core Concepts

`autosurgeon` bridges the gap between Rust types and the Automerge document
model using two main traits:

- **`Reconcile`**: Updates the Automerge document to match a Rust struct.
- **`Hydrate`**: Reads data from the Automerge document into a Rust struct.

## Surgical Mutations

A "surgical" mutation targets a specific part of the document rather than
reconciling the entire root. This is useful when you want to update a specific
object or a subset of fields without touching the rest of the document.

### Targeting Specific Places with `reconcile_prop`

The `reconcile_prop` function allows you to reconcile a Rust struct into a
specific property of a specific object in the Automerge document.

**Generic Signature:**

```rust
pub fn reconcile_prop<'a, D, R, O, P>(
    doc: &mut D,
    obj: O,
    prop: P,
    value: R
) -> Result<(), ReconcileError>
where
    D: Doc,                 // The Automerge document (or transaction)
    R: Reconcile,           // The value to reconcile
    O: AsRef<ObjId>,        // The object ID containing the property
    P: Into<Prop<'a>>,      // The property key (string) or index (number)
```

**Example:**

Assume you have a document structure like this:

```json
{
  "users": {
    "user_123": {
      "name": "Alice",
      "preferences": {
        "theme": "light",
        "notifications": true
      }
    }
  }
}
```

And you want to update just the **preferences** for `user_123`.

```rust
use autosurgeon::{Reconcile, Hydrate, reconcile_prop};
use automerge::ReadDoc;
use automerge::ObjType;

#[derive(Debug, Clone, Reconcile, Hydrate, PartialEq)]
struct Preferences {
    theme: String,
    notifications: bool,
}

fn update_user_preferences(doc: &mut automerge::AutoCommit, user_id: &str, prefs: Preferences) {
    // 1. Navigate to the specific object you want to update.
    // In a real app, you might already have the ObjId of "user_123" cached or looked up.
    // Here we find it manually for demonstration.
    let users_obj: automerge::ObjId = doc
        .get(automerge::ROOT, "users")
        .expect("Failed to read from document") // Result: Internal automerge errors
        .expect("'users' key should exist")     // Option: The key might not be present
        .1; // The tuple is (Value, ObjId). We want the ObjId.

    let user_obj: automerge::ObjId = doc
        .get(&users_obj, user_id)
        .expect("Failed to read from 'users' map")
        .expect("User key should exist")
        .1; // The tuple is (Value, ObjId). We want the ObjId.

    // 2. Reconcile the "preferences" property of that user object
    reconcile_prop(doc, user_obj, "preferences", prefs).unwrap();
}
```

### Partial Updates (Partial Reconciliation)

`autosurgeon` supports "partial updates" naturally because `Reconcile`
implementations for structs typically only touch the fields defined in the
struct. If the target object in the document has _extra_ fields that are not in
your Rust struct, `autosurgeon` will generally leave them alone (unless you are
replacing the entire map/object in a way that implies exclusivity, but the
standard derive macro works field-by-field).

To perform a partial update, define a struct that contains **only the fields you
want to change**.

**Example:**

Going back to our user example, suppose you only want to update the `theme` and
leave `notifications` alone.

1.  Define a "Patch" struct.
2.  Use `reconcile_prop` (or `reconcile`) with that patch.

```rust
#[derive(Reconcile)]
struct ThemePatch {
    theme: String,
}

fn update_theme_only(doc: &mut automerge::AutoCommit, user_obj: &automerge::ObjId, new_theme: String) {
    // This will update "theme" within the "preferences" object
    // "notifications" will remain untouched because ThemePatch doesn't know about it.
    let patch = ThemePatch { theme: new_theme };

    // Note: We are reconciling into the "preferences" property of the user object
    reconcile_prop(doc, user_obj, "preferences", patch).unwrap();
}
```

**Warning:** This works because the standard `Reconcile` derive macro generates
code that performs individual `put` operations for each field. It does **not**
wipe the object clean before writing.

## Hydrating from Specific Places

Just as you can write to a specific place, you can read from one.

### `hydrate_prop`

Use `hydrate_prop` to read a value from a specific property of an object.

```rust
use autosurgeon::{hydrate_prop, Hydrate};

#[derive(Debug, Hydrate, PartialEq)]
struct Preferences {
    theme: String,
    notifications: bool,
}

fn get_user_preferences(doc: &automerge::AutoCommit, user_obj: &automerge::ObjId) -> Option<Preferences> {
    // Hydrate the "preferences" struct specifically from the "preferences" key of the user object
    hydrate_prop(doc, user_obj, "preferences").ok()
}
```

### `hydrate_path`

The top-level `hydrate` function always initiates hydration from the document
root (`automerge::ROOT`). `autosurgeon` does not provide a direct mechanism to
hydrate an arbitrary `ObjId` in isolation, as the hydration process relies on
property traversal for correct type mapping.

To hydrate data located deep within the document, you have two primary options.

#### Option 1: Traverse and Hydrate Property

Manually navigate to the **parent** object of the data you want, then hydrate
the specific property.

- **Benefit:** Efficient if you already have the parent's `ObjId` (e.g., from a
  previous operation).
- **Drawback:** Requires verbose manual `doc.get` calls if starting from the
  root.

```rust
use autosurgeon::{hydrate_prop, Hydrate};
use automerge::ReadDoc;

fn get_user_preferences_via_traversal(
    doc: &automerge::AutoCommit,
    user_id: &str
) -> Result<Preferences, autosurgeon::HydrateError> {
    // 1. Manually traverse to the user object (the parent of "preferences")
    let users_obj: automerge::ObjId = doc.get(automerge::ROOT, "users")
        .map_err(|e| autosurgeon::HydrateError::Automerge(e))?
        .expect("users map missing")
        .1;

    let user_obj: automerge::ObjId = doc.get(&users_obj, user_id)
        .map_err(|e| autosurgeon::HydrateError::Automerge(e))?
        .expect("user missing")
        .1;

    // 2. Hydrate the "preferences" property from the user object
    hydrate_prop(doc, &user_obj, "preferences")
}
```

#### Option 2: Hydrate Path

Use `hydrate_path` to specify a property path from the root (or another known
object).

- **Benefit:** Concise for deep lookups; avoids manual traversal boilerplate.
- **Drawback:** Does not expose intermediate `ObjId`s if they are needed for
  other operations.

```rust
use autosurgeon::{hydrate_path, Hydrate};

fn get_user_preferences_via_path(
    doc: &automerge::AutoCommit,
    user_id: &str
) -> Result<Preferences, autosurgeon::HydrateError> {
    // Construct the path: root -> "users" -> <user_id> -> "preferences"
    let path = vec![
        "users".into(),
        user_id.into(),
        "preferences".into()
    ];

    // Hydrate the value at the end of the path
    // hydrate_path returns Result<Option<T>, ...>
    let prefs = hydrate_path(doc, &automerge::ROOT, path)?
        .expect("preferences not found");

    Ok(prefs)
}
```

## Summary API Table

| Goal                | API Function                           | Context                                           |
| :------------------ | :------------------------------------- | :------------------------------------------------ |
| **Full Root Sync**  | `reconcile(doc, &struct)`              | Syncs struct to `automerge::ROOT`.                |
| **Surgical Update** | `reconcile_prop(doc, obj, prop, &val)` | Syncs struct to `doc.get(obj, prop)`.             |
| **Partial Update**  | `reconcile_prop` + Subset Struct       | Define a struct with _only_ the fields to update. |
| **Read Value**      | `hydrate_prop(doc, obj, prop)`         | Reads specific value into struct.                 |
| **Deep Read**       | `hydrate_path(doc, obj, path)`         | Reads value at specific path.                     |
| **Deep Write**      | _Custom Helper_                        | See below.                                        |

### `ensure_path` (Custom Helper)

Instead of a monolithic `reconcile_path`, it is often better to use a helper
that ensures a path of objects exists and returns the final `ObjId`. You can
then use this `ObjId` for multiple operations (hydration or reconciliation).

```rust
fn ensure_path(
    doc: &mut automerge::AutoCommit, // or generic D: Transactable
    root: &automerge::ObjId,
    path: Vec<&str>,
) -> Result<automerge::ObjId, anyhow::Error> {
    let mut current = root.clone();
    for key in path {
        let val = doc.get(&current, key).map_err(|e| anyhow::anyhow!(e))?;
        current = match val {
            Some((automerge::Value::Object(_), id)) => id,
            None => doc.put_object(&current, key, automerge::ObjType::Map).map_err(|e| anyhow::anyhow!(e))?,
            _ => return Err(anyhow::anyhow!("Path key '{}' is not an object", key)),
        };
    }
    Ok(current)
}
```

**Usage:**

```rust
let tasks_map = ensure_path(&mut doc, &automerge::ROOT, vec!["tasks"])?;
reconcile_prop(&mut doc, tasks_map, "task_123", &my_task)?;
```
