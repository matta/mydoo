# State Code Walk for Systems Programmers

## Overview

Here is a walk through of the state management system in Dioxus. Zero knowledge of Dioxus, or any other Web Framework is assumed. We will start from the ground up, and provide a bottom up view of the system.

## Key Concepts

- **State**: Data that is owned by a component and can be changed over time.
- **Signals**: A reactive primitive that allows you to create and manage state.
- **Props**: Data that is passed to a component from its parent.
- **Context**: Data that is passed to a component from its ancestors.
- **Hooks**: Functions that allow you to use state and other features in your components.

## Key Files

- **`packages/core/src/lib.rs`**: The main entry point for the Dioxus core library.
- **`packages/core/src/signals.rs`**: The signals system.
- **`packages/core/src/state.rs`**: The state management system.
- **`packages/core/src/props.rs`**: The props system.
- **`packages/core/src/context.rs`**: The context system.
- **`packages/core/src/hooks.rs`**: The hooks system.

## Foundational Concepts

The following internal Dioxus concepts are referenced throughout the Signals architecture and serve as prerequisites:

- **Scope (`ScopeId`)**: The fundamental unit of ownership in Dioxus. Each component instance corresponds to a `Scope`, which owns the lifetime of hooks and signals created within it.
- **Task**: An asynchronous unit of work (future) spawned by the Runtime. Tasks can read signals and will wake up when those signals change.
- **Reactive Context**: A thread-local tracking mechanism that answers "Who is reading this signal right now?". It allows a Signal to automatically register the current Scope, Memo, or Task as a subscriber.
- **Runtime**: The global orchestrator that manages the tree of Scopes, the Task scheduler, and the Event Loop. Signal operations (read/write) rely on the Runtime to identify the current context.

## Scopes

Scopes are the fundamental unit of ownership and identity in Dioxus. Each component instance corresponds to a `Scope`.

#### Scopes: Execution Context & State Container

A `Scope` (`packages/core/src/scope_context.rs`) serves as the runtime representation of a component instance. It acts as the execution context, managing the lifecycle of the component's state, children, side-effects, and asynchronous tasks. It ensures that all resources associated with a component are allocated, accessible, and deallocated as a coherent unit.

#### Scopes: Internal Anatomy

The `Scope` struct is a dense intersection of systems. Its fields reveal its responsibilities:

```rust
pub(crate) struct Scope {
    // Identity & Hierarchy
    pub(crate) id: ScopeId,
    pub(crate) parent_id: Option<ScopeId>,
    pub(crate) height: u32,

    // State & Resources
    pub(crate) hooks: RefCell<Vec<Box<dyn Any>>>,
    pub(crate) shared_contexts: RefCell<Vec<Box<dyn Any>>>,
    pub(crate) spawned_tasks: RefCell<FxHashSet<Task>>,

    // Lifecycle & Rendering
    pub(crate) render_count: Cell<usize>,
    pub(crate) before_render: RefCell<Vec<Box<dyn FnMut()>>>,
    pub(crate) after_render: RefCell<Vec<Box<dyn FnMut()>>>,
    pub(crate) status: RefCell<ScopeStatus>,
    suspense_boundary: SuspenseLocation,
}
```

#### Scopes: Memory Management & Lifetimes

- **Arena-like Ownership**: The Scope acts as a localized arena. When the Runtime drops a `Scope`, it drops everything inside it. This guarantees that component unmounting is synonymous with resource cleanup.
- **Drop Order Invariant**: The `Scope` enforces a strict drop order: Hooks are dropped BEFORE Contexts. This allows a Hook's `Drop` implementation to safely access shared contexts (e.g., a signal unregistering itself from a global store) before those contexts are deallocated.

#### Scopes: Dynamic Typing and Runtime Checks

Dioxus needs to store variables of many different types (integers, strings, structs) in a single list. In statically typed languages like Rust, this requires a technique called **Type Erasure**.

1.  **State managed by Index (Hooks)**:
    - **Storage**: `hooks`. A list of pointers to data of any type (`Box<dyn Any>`).
    - **Identification**: Variables are identified by their **position**. The 3rd variable declared in your function is always stored at index `2`.
    - **Safety Checks**: The runtime checks the type when accessing data. If index `2` holds an Integer but your code tries to read a String, the program will safely abort (panic) rather than corrupting memory.

- **Context Resolution**: See [Shared Data Details](#scopes-shared-data-context-system) below.

#### Scopes: Background Tasks

- **Task Tracking**: The Scope keeps a list of all background operations (async tasks) started by this component.
- **Automatic Cleanup**: When a component is removed, the Scope is destroyed. It immediately cancels all its running background tasks. This prevents a classic bug where a task finishes later and tries to write to memory that no longer exists (Use-After-Free).
- **Waiting for Data (Suspense)**: The Scope tracks if it is currently waiting for a slow operation (like a download) to finish, allowing the rendering engine to wait before showing the component.

### Scopes: Shared Data (Context System)

This system allows data to be shared across the tree of components without passing arguments manually through every function (a pattern known as Dependency Injection).

#### Scopes: Context Data Structures

- **Storage**: Each Scope holds a list of shared objects.
- **Lookup by Type**: Unlike a Hash Map that uses strings for keys, this system uses the **Data Type** itself as the key. You can store one `User` object and one `Theme` object. This guarantees you never read the wrong type of data.

#### Scopes: Context Search Algorithm

When a component asks for a specific type of data (e.g., `User`), the system performs a simple linear search up the tree:

1.  **Check Local**: Look in the current Scope's list. If a `User` object is found, return a copy.
2.  **Move Up**: If not found, move to the **Parent Scope**.
3.  **Repeat**: Continue checking parents.
4.  **Found or Fail**: If found, return it. If the Root Scope is reached without finding it, return nothing.

This algorithm's performance depends on how deep the component is in the tree (Linearly Proportional to Depth, or `O(d)`).

## Signals

Signals are the primary state primitive in Dioxus. They provide a `Copy` handle to heap-allocated, mutable state with automatic dependency tracking (reactivity).

### Signals: Ground Up Architecture

The Signals system is built in layers, moving from raw memory management to high-level reactive APIs.

#### Signals: Level 1: `GenerationalBox` (Memory Management)

At the very bottom is `GenerationalBox` (found in `packages/generational-box`).

- **Problem**: In UI graphs, components come and go. Passing raw references (`&RefCell<T>`) is dangerous because the owner might be dropped while a child holds a reference (lifetimes). `Rc<RefCell<T>>` solves this but introduces reference cycles and leaks.
- **Solution**: A Generational Arena.
  - Data is stored in a central "Store" tied to a `Scope`.
  - Handles (`GenerationalBoxId`) are lighter than `Rc`.
  - Accessing an ID checks if the "generation" matches. If the original owner (Scope) died and the slot was reused, generic access fails safely instead of reading garbage.
- **Backing**: Can be `UnsyncStorage` (RefCell-like, for single-threaded WASM) or `SyncStorage` (RwLock-like, for threads).
- **Drawbacks**: By using handles (`Copy` IDs) instead of references, we detach the lifetime of the handle from the lifetime of the data. The compiler can no longer guarantee that the data exists when you try to use the handle.
  - If you hold a `GenerationalBoxId` (Signal) after its owner (Component Scope) has been dropped, the data is gone.
  - Accessing it results in a runtime error (panic or `Result::Err`), whereas strictly borrowed Rust code would have failed to compile.
  - We effectively trade compile-time lifetime analysis for runtime generation checks (preventing use-after-free, but allowing logic errors).

#### Signals: Level 2: `CopyValue<T>` (`packages/signals/src/copy_value.rs`)

`CopyValue` is a thin wrapper around a `GenerationalBox`.

- **Role**: It gives a user-friendly API to the raw box (`read()`, `write()`, `set()`).
- **Characteristics**: It implements Shared Reference semantics (similar to `Rc<RefCell<T>>`).
  - The `CopyValue` struct is small and `Copy`—it is just a handle.
  - Copying the handle _does not_ clone the data. All copies point to the same single value in the store.
- **No Reactivity**: It does _not_ track subscribers. It just holds data.

#### Signals Level 3: `SignalData<T>` (`packages/signals/src/signal.rs`)

To enable reactivity, the raw value `T` is not stored directly in the `CopyValue`. Instead, it is wrapped in `SignalData<T>`, which associates the value with its dependency graph.

```rust
pub struct SignalData<T> {
    pub(crate) subscribers: Arc<Mutex<HashSet<ReactiveContext>>>,
    pub(crate) value: T,
}
```

- **Role**: This structure functions as the Reactive Payload.
  - **Integration with Lower Levels**: `SignalData<T>` is the concrete type stored inside the `GenerationalBox` (Level 1). The `CopyValue` (Level 2) holds a handle to this `SignalData<T>`, making it effectively a `CopyValue<SignalData<T>>`.
  - **Functionality**: It pairs the actual data (`value`) with the graph edges (`subscribers`). The `subscribers` set tracks every Component or Effect that depends on this value.
- **Usage**:
  - **Internal Only**: Dioxus applications **do not** use `SignalData` directly. It is an internal implementation primitive used to build `Signal<T>`.
  - **Invariants**: The `subscribers` field uses an internal `Arc<Mutex<...>>` separate from the `GenerationalBox`'s lock. This fine-grained locking prevents deadlocks by allowing subscriber list updates (e.g., when dropping a write lock) without holding the lock on the `value` itself.

#### Signals Level 4: `Signal<T>` (`packages/signals/src/signal.rs`)

The primary user-facing type. `Signal<T>` wraps `CopyValue<SignalData<T>>`.

- **Role**: `Signal<T>` acts as the main entry point for state in Dioxus applications. It intercepts `read()` and `write()` operations to drive the reactivity system.
  - **Foundational Primitive**: `Signal<T>` is not just for user code; it serves as the building block for other high-level state primitives. `Memo<T>`, `GlobalSignal<T>`, and `Resource<T>` are all built on top of or composed of `Signal<T>`.
- **Usage Patterns**:
  - **Creation**: Typically created via `use_signal(|| value)` inside a component. This hook allocates the signal once and persists it across renders.
  - **Access**: Accessed via dereferencing (`*sig.read()`) or function call syntax (`sig()`) in `rsx!` for automatic subscription.
- **Rules & Invariants**:
  - **Copy Semantics**: signals are `Copy`. Passing a signal to a child component passes a specific _handle_ to the same shared memory.
  - **Scope-Bound**: A signal created with `use_signal` is owned by the component's scope. It cannot outlive the component.
- **Best Practices**:
  - **Pass Signals, Not Values**: When moving state to a child component, pass the `Signal<T>` handle itself as a prop. This allows the child to read and modify the state without causing the parent component to re-render.
  - **Local Mutability**: Use signals for local, mutable state. For immutable or top-down data flow, prefer plain props.

### Signals: How it Works Under the Hood

#### Signals: The Reactive Loop

1.  **Subscription (Read)**:
    When you call `signal.read()` (or deref it):
    - Implementation: `packages/signals/src/read.rs` (via `Readable` trait).
    - It locks the inner `GenerationalBox`.
    - It checks `ReactiveContext::current()`. This is a thread-local variable set by Dioxus when running a Component or Effect.
    - If a context exists, it adds that context to `SignalData.subscribers`.
    - It returns a `Ref<'a, T>`.

2.  **Notification (Write)**:
    When you call `signal.write()`:
    - Implementation: `packages/signals/src/write.rs` (via `Writable` trait).
    - It locks the inner storage (RefCell borrow_mut).
    - It returns a `WriteLock<'a, T>`.
    - **Drop Magic**: The `WriteLock` implements `Drop`. When the guard goes out of scope:
      - It takes the lock on `subscribers`.
      - It iterates through all subscribers.
      - Calls `subscriber.mark_dirty()`.
      - The Dioxus Virtual DOM picks up these dirty scopes and re-renders them next frame.

#### Signals: Memos (`packages/signals/src/memo.rs`)

`Memo<T>` is a computed signal (`Signal::memo(|| a() + b())`).

- It creates a **new** `ReactiveContext` for itself.
- It provides a closure to that context.
- When `a()` is read inside the closure, `a` adds the Memo's context to its subscribers (not the component's).
- When `a()` writes, it marks the Memo dirty.
- The Memo logic (running in a `spawn_isomorphic` task) catches this signal and re-runs the closure to update its internal value.

#### Signals: Update Deduplication

Does Dioxus check if the value actually changed before triggering an update?

1.  **Signals (`use_signal`)**: **No**.
    Calling `.set(value)` triggers an update _unconditionally_, even if the new value is identical to the old one (e.g., setting `5` to `5`).
    - _Why?_ Equality checks (`PartialEq`) can be expensive for large structs. Dioxus assumes if you called write, you meant it.
    - _Workaround_: If you need deduplication for a signal, check manually:
      ```rust
      if *sig.peek() != new_value { sig.set(new_value); }
      ```

2.  **Memos (`use_memo`)**: **Yes**.
    Memos are designed for derived state. After the memo re-calculates its value, it checks `PartialEq` against the previous value. If they are equal, it **aborts** the update, and downstream subscribers are **not** notified.

3.  **Selectors (`use_set_compare`)**: **Yes**.
    These are explicitly designed to filter updates based on equality.

### Signals: Key Trade-offs & Design Decisions

- **Copy Handles vs References**:
  - _Decision_: Signals are `Copy`.
  - _Why_: Ergonomics. No need to `clone()` explicitly to pass into closures. "Just Works" in `rsx!`.
  - _Trade-off_: Runtime overhead of RefCell/RwLock checks. Potential for `BorrowError` panics if you read an active write.

- **Generational Lifetimes**:
  - _Decision_: Signals are owned by a Scope.
  - _Why_: Prevents memory leaks. When a Component drops, its signals are cleaned up.
  - _Trade-off_: You cannot simply extract a Signal out of a component and keep it alive forever unless you create it in the Root scope or use `GlobalSignal`.

- **Coarse-Grained Updates (Component Level)**:
  - _Decision_: By default, reading a signal subscribes the _entire component_.
  - _Why_: Simpler mental model (like React). No need for fine-grained compiled "islands" (like SolidJS) complexity in the compiler.
  - _Trade-off_: If a component is huge and reads a signal that changes often, the whole component diffs. (Mitigation: Break up components, use Memos).

### Signals: Programmer Invariants (Correctness)

To use Signals correctly, you must honor the Runtime Borrowing Rules:

1.  **No Overlapping Read/Write**:
    - `let r = sig.read(); sig.write();` -> Panic.
    - _Fix_: Drop the read before writing, or use `peek()`.

2.  **No Writes across Await**:
    - `let mut w = sig.write(); some_future.await;`
    - _Why_: The write lock is held while the future pauses. If the future's executor tries to read the signal (e.g., rendering) while the lock is held:
      - **Web** (`UnsyncStorage`): It will panic (RefCell check failed).
      - **Desktop/Server** (`SyncStorage`): It will deadlock (RwLock blocks waiting for itself).
    - _Fix_:
      ```rust
      let val = *sig.read();
      let new_val = async_op(val).await;
      sig.set(new_val);
      ```

3.  **Scope Safety**:
    - Don't send a hook-created Signal to a thread/place that outlives the component, unless you accept it will eventually become invalid (and panic on access).

### Signals: Performance & Optimization

- **Use `peek()`**:
  - If you are reading a value inside an event handler (e.g., `onclick`), use `signal.peek()`.
  - _Why_: Interaction handlers shouldn't subscribe to the value. You don't want the component to re-render just because you clicked a button that read the current count.

- **Signals in Props**:
  - Pass `Signal<T>` as a prop, not `T`.
  - _Why_: The parent doesn't need to read it (and subscribe). The child reads it. This prevents the Parent from re-rendering when the value changes; only the Child updates.

- **Global Signals**:
  - For app-wide state, use `GlobalSignal`. It's initialized lazily and accessible everywhere, avoiding prop drilling.

### Signals: Standalone Usage

- **Dioxus Dependency**: Signals are deeply integrated with the Dioxus Runtime. They are not a standalone library like `atomic-refcell` or `crossbeam`. They rely on the runtime for two critical services:
  1.  **Ownership (Memory Safety)**: `Signal::new` needs an `Owner` to attach the data to. This `Owner` is typically the current Component Scope. Without a Runtime, there is no "current scope," so the signal doesn't know when to free its memory.
  2.  **Topology (Reactivity)**: The reactivity system relies on identification. When `signal.read()` is called, it asks the Runtime "Who is currently running?". The Runtime responds with a `ScopeId` or `Task`. Without a Runtime, the signal has no subscribers to track.
- **Can I use them outside Dioxus?**: Generally, **no**. `Signal::new` attempts to access the `Runtime::current()`. If no runtime is present, it panics.
- **Unit Testing**: You **can** unit test code that uses signals, but you must provide a mock environment. The simplest way to spin up a Runtime is via `VirtualDom::new`.
  - _Why VirtualDom?_: While `Runtime` is the actual requirement, `VirtualDom` is the public API that initializes the Runtime, Root Scope, and Event Loop correctly.
  - _Example_:
    ```rust
    #[test]
    fn test_signal_logic() {
        // Initialize a VirtualDom to provide the necessary Runtime
        let mut dom = VirtualDom::new(|| {
            let mut sig = use_signal(|| 0);
            sig += 1;
            rsx! { "{sig}" }
        });
        // Run the virtual dom to execute the code
        dom.rebuild_in_place();
    }
    ```
- **Non-Dioxus Apps**: It is not recommended to use Dioxus Signals as a general-purpose state management library for non-Dioxus applications (like a pure CLI tool or a game engine using a different renderer) due to the heavy runtime requirement.

## Hooks

In Dioxus, components are designed to be pure functions. Conceptually, they take arguments (Props) as input and return a UI description as output. Pure functions are predictable—they have no memory of previous executions and always produce the same result for the same input.

Hooks are the tool that allows these stateless functions to become interactive applications. They provide:

1.  Reactivity: The automatic synchronization of the User Interface with application state. When data from a hook changes, the Dioxus Runtime detects this and re-executes the necessary component functions to produce the new UI.
2.  Side Effects: The ability to change the state of the program or the outside world (like making a network request or changing the page title).
3.  Persistence: The ability to maintain state across multiple component function invocations.

### Hooks: Ground Up Architecture

How does a function "remember" data between calls if it doesn't have local variables? It uses a hidden list attached to the component.

#### Level 1: Storage & Identity (`packages/core/src/scope_context.rs`)

Hooks are stored in the component's `Scope`, and identified by their execution order within the component function.

This system includes dynamic type safety. When a hook tries to retrieve its state, the runtime verifies that the type stored at the current index matches the type requested by the hook function. If the types do not match—for example, if a code change alters the hook order so that an Integer hook attempts to read a String value—the system will panic.

To map a hook call to its corresponding state, the system relies on the sequence of execution. The component function's first hook call retrieves the first hook's value, the second call retrieves the second hook's value, and so on. As the component executes, a pointer tracks the current position in this list, advancing sequentially with each hook invocation. This list is stored in the Scope as a vector of generic pointers (`Box<dyn Any>`), allowing it to hold state of diverse types.

You might observe that hook functions do not accept the Scope as an argument. How do they locate the correct state list?

The system uses Thread Local Storage to pass this context implicitly. Before the runtime calls a component function, it sets a global variable (specifically, a thread-local variable) to point to the component's Scope. When a hook executes, it reads this global variable to find the active Scope. This technique allows the component API to remain clean, avoiding the need to pass context arguments through every function call.

#### Level 2: The Primitive (`use_hook`) (Internal)

Almost all hooks (`use_signal`, `use_context`, etc.) eventually call the internal `Scope::use_hook` method.

```rust
pub fn use_hook<State>(initializer: impl FnOnce() -> State) -> State {
    // 1. Get the current index and increment the cursor
    let idx = scope.hook_index.get();
    scope.hook_index.set(idx + 1);

    // 2. Check if a hook already exists at this index
    if let Some(existing) = scope.hooks.get(idx) {
        // 3. Downcast and return (Fast Path)
        return existing.downcast_ref::<State>().clone();
    }

    // 4. Initialization (Slow Path - First Render Only)
    let value = initializer();
    scope.hooks.push(Box::new(value.clone()));
    value
}
```

_Note: This is a simplified view. The actual implementation handles some edge cases and hot-reloading checks._

#### Level 3: Built-in Hooks (`packages/hooks`)

While `use_hook` is the low-level primitive (like a system call), Dioxus provides a "standard library" of common abstractions to make development easier. These are just normal functions that call `use_hook` internally.

The library includes many such helpers to cover common requirements:

**State Management**

- `use_signal`: Creates a piece of mutable state.
- `use_context`: Consumes data from a parent component (Dependency Injection).
- `use_root_context`: Consumes data specifically from the root of the application.
- `use_reactive`: Creates a state object that triggers updates on deep changes (Deep Mutability).

**Async & Side Effects**

- `use_resource`: Manages a Future that produces a value (like fetching data from a server).
- `use_future`: Starts a background Future without the reactive tracking of a Resource.
- `use_coroutine`: Starts a separate task that can process messages (Actor Model).
- `use_effect`: Runs a function automatically when dependencies change (Side Effects).
- `use_on_destroy`: Registers a callback to run when the component is removed (Cleanup).
- `use_action`: Wraps an event handler or action dispatch.
- `use_after_suspense_resolved`: Executes logic only after all async background tasks have finished.

**Performance (Memoization)**

- `use_memo`: Caches the result of a usage-intensive calculation.
- `use_callback`: Caches a function definition to prevent unnecessary updates in child components.
- `use_set_compare`: Optimizes updates by checking if a new value is deeply equal to the old value before writing.

**Low-Level System Tools**

- `use_waker`: Provides a handle to manually wake up (re-render) the component.
- `use_hook_did_run`: checks if a specific hook has already executed in the current render cycle.

### Hooks: use_signal

This hook allocates a piece of mutable memory that persists across renders. It also implements automatic dependency tracking.

Dioxus uses a signal-based architecture. While `use_hook` is the primitive for storage, `use_signal` is the primitive for reactivity. It creates a value that, when written to, automatically triggers an update in any component (or memo) that reads it.

**How Tracking Works**:
You do not need to manually tell Dioxus which data a component uses. When your component code runs and reads a signal (e.g. by printing `{count}`), the runtime "sees" this read operation. It automatically adds the current component to the signal's list of subscribers. Later, when you change the signal's value, the system looks at this list and re-runs only the components that actually used the data.

**Classification**: Fundamental. It is the primary way to create state that drives the UI.

**Best Practices**:
Use this for local state that belongs to the component, like form input or toggle switches.

```rust
let mut count = use_signal(|| 0);
rsx! {
    // Reading '{count}' subscribes this component.
    // Writing 'count += 1' notifies subscribers.
    button { onclick: move |_| count += 1, "{count()}" }
}
```

_Note: `count` is a struct, not an integer. Rust's Operator Overloading features allow us to treat it like a number syntax-wise (`+=`), but under the hood, it calls `.write()` and updates the internal value._

### Hooks: use_context

This hook implements the Dependency Injection pattern. Unlike other hooks which rely on execution order to find their own state, `use_context` searches up the component tree to find data provided by a parent. It identifies this data by its type (e.g., looking for a `Theme` struct).

**Classification**: Common.

**Best Practices**:
Use this for shared state like color themes, user sessions, or settings that many components need to access.

```rust
// Parent Component: Provides the data
fn App() -> Element {
    // "Inject" the Theme into the tree
    use_context_provider(|| Signal::new(Theme::Dark));
    rsx! { Profile {} }
}

// Intermediate Component: Doesn't need to know about Theme
fn Profile() -> Element {
    rsx! { ThemeButton {} } // Just renders children
}

// Descendant Component: Consumes the data
fn ThemeButton() -> Element {
    // Finds the 'Theme' signal provided by App
    let theme = use_context::<Signal<Theme>>();
    rsx! { "Current theme: {theme}" }
}
```

### Hooks: use_root_context

A specialized version of dependency injection designed for libraries. It enables the **Singleton Pattern** within the component tree.

Unlike `use_context` which fails if the data isn't found, `use_root_context` checks the root of the app. If the data exists, it returns it. If not, it runs the provided initializer, installs the data at the root, and _then_ returns it.

**Classification**: Specialized (Library Development).

**Best Practices**:
Use this when building libraries (like a Router or Logger) to ensure a single global instance exists, without forcing the user to manually wrap their app in a `<Provider>`.

```rust
// A custom hook for a library
fn use_logger() -> Logger {
    // If a Logger exists at the root, return it.
    // If not, create a new Logger, attach it to the root, and return it.
    use_root_context(|| Logger::new())
}
```

### Hooks: use_reactive

This hook acts as an adapter. It takes plain, non-reactive data (like a simple integer or string passed from a parent) and "activates" it so it can trigger updates.

**How It Works**:
On every render, `use_reactive` compares the current value of the data to the previous value. If it has changed (e.g. `1` becomes `2`), it updates an internal signal, which then triggers any downstream effects or memos.

**Classification**: Specialized (Adapter).

**Best Practices**:
Use this when you have a `use_effect` that depends on a simple Prop (not a Signal) and you want the effect to re-run only when that Prop changes.

```rust
// 'props.name' is just a String, not a Signal.
// We wrap it so the effect runs when the name changes.
use_effect(use_reactive(&props.name, |name| {
    println!("Name changed to: {}", name);
}));
```

### Hooks: use_resource

This hook integrates asynchronous tasks (Futures) with the component lifecycle. It starts a background task to compute a value (like fetching data) and provides a handle to track its status (loading, finished, error).

**Classification**: Common. The standard way to load data from a server.

**Best Practices**:
Use `.read()` to check if the data has arrived. Dioxus handles the "loading" state automatically for you.

```rust
let user = use_resource(move || async move { fetch_user().await });
match &*user.read_unchecked() {
    Some(Ok(u)) => rsx! { "Hello {u.name}" },
    _ => rsx! { "Loading..." }
}
```

### Hooks: use_future

Similar to `use_resource`, this spawns a background thread (green thread) for async work, but it does not automatically provide the "loading/error" state wrapping. It is a raw Future handle.

**Classification**: Specialized. Use this when you need fire-and-forget async tasks or want to manage the state manually.

### Hooks: use_coroutine

This hook implements the Actor Model. It spawns a separate task that listens for messages on a channel. You can send messages to it, and it processes them sequentially.

**Classification**: Specialized. Excellent for complex state machines or WebSocket connections.

```rust
let chat = use_coroutine(|mut rx: UnboundedReceiver<String>| async move {
    while let Some(msg) = rx.next().await { println!("Sent: {msg}"); }
});
// chat.send("Hello".to_string());
```

### Hooks: use_effect

This hook allows you to run a side effect (an action that changes the world outside the component) after the component renders. It typically runs when the component mounts or when dependencies change.

**Classification**: Common.

**Best Practices**:
Use this for things like changing the document title, logging, or interacting with browser APIs that are not part of Dioxus.

```rust
use_effect(move || {
    document::set_title("New Title");
});
```

### Hooks: use_on_destroy

This hook registers a cleanup function (destructor) that runs immediately before the component is removed from the screen.

**Classification**: Specialized. Useful for cleaning up external libraries or timers.

### Hooks: use_action

This hook acts as a state machine for asynchronous event handlers. It wraps a user-defined async function and tracks its lifecycle.

Unlike a standard event handler (which is fire-and-forget), `use_action` gives you a handle to query the current status of the operation: **Pending**, **Ready** (with a value), or **Errored**.

**Classification**: Specialized (UI Feedback).

**Best Practices**:
Use this for form submissions or API calls triggered by user interaction, where you want to disable the button or show a loading spinner while the action runs.

```rust
let submit = use_action(move |_| async move {
    // Simulate an API call
    Ok("Success")
});

rsx! {
    button {
        // Disable button while the action is running
        disabled: submit.pending(),
        onclick: move |_| submit.call(),
        "Submit"
    }
    if let Some(Ok(val)) = submit.value() {
        "Result: {val}"
    }
}
```

### Hooks: use_after_suspense_resolved

This hook delays execution until all other background "resource" tasks within the component tree have finished loading. It acts as a barrier synchronization primitive.

**Classification**: Specialized.

### Hooks: use_memo

This hook implements memoization (caching). It stores the result of a calculation and only re-calculates it if the input dependencies change.

**The Alternative: Computed State**
Most of the time, you do not need `use_memo`. You can simply calculate the value directly in the function body.

```rust
// This runs every time the component renders, which is fine for fast math.
let double_count = count() * 2;
```

**Classification**: Optimization.

**Best Practices**:
Use `use_memo` only when the calculation is expensive (like sorting a list of 10,000 items). For simple logic, the overhead of checking the cache is actually slower than just doing the math.

```rust
// Memoization (Optimization)
// Only re-runs calculate_prime() when 'count' changes.
let expensive = use_memo(move || calculate_prime(count()));
```

### Hooks: use_callback

This hook caches a function pointer (closure) so that it remains stable across renders. This prevents child components from re-rendering just because a new "function" was created.

**Classification**: Optimization.

### Hooks: use_set_compare

This hook creates a comparison object that tracks a source value. It returns a `SetCompare<T>` handle.

**Usage**: You are intended to pass this handle as a prop to child components. This allows the parent to manage the source of truth while enabling many child components to efficiently check "Is the value equal to X?" without the parent having to manually re-render them.

**Classification**: Optimization.

```rust
// Create the comparison object
// 'compare' is type SetCompare<usize>
let compare = use_set_compare(move || selected_id());
// Pass 'compare' to children...
```

### Hooks: use_set_compare_equal

This hook subscribes to the comparison object created by the parent's call to `use_set_compare`. It returns a boolean signal that is true only if the parent's source value equals the specific value provided to this hook.

The system is smart: when the parent source changes, Dioxus only re-renders the specific components where the equality result effectively flips (e.g. from true to false).

**Example: Efficient List Selection**
In a list of items, we only want to re-render the row being selected and the row being deselected.

```rust
// Parent component
let selected_id = use_signal(|| 0);
let compare = use_set_compare(move || selected_id());

// Child component (Rendered 10,000 times)
// checks if 'my_id' matches the 'selected_id'.
// Only re-renders if this specific result flips.
let is_selected = use_set_compare_equal(my_id, compare);
```

### Hooks: use_waker

This gives you a raw handle to the component's update function. Calling it manually forces the component to re-render, bypassing the normal reactive flow.

**Classification**: Low-Level. Avoid using this unless building a custom hook library.

### Hooks: use_hook_did_run

This is a lifecycle auditor used primarily for debugging custom hooks. It helps you verify if a specific hook executed during the current render cycle.

**How It Works**:

1. Before the component renders, it resets a flag to `false`.
2. When this hook runs, it sets the flag to `true`.
3. After the render completes, it calls your callback with the final status.

**Classification**: Internal Tooling.

**Usage**:
Library authors use this to ensure that complex custom hooks are behaving correctly and not being accidentally skipped.

```rust
fn use_my_complex_logic() {
    // Debugging check: will print "Did run: true" if this line is reached.
    use_hook_did_run(|did_run| println!("Did use_my_complex_logic run? {}", did_run));

    // ... rest of hook logic
}
```

### Hooks: Design Decisions & Trade-offs

- **Implicit Identity (Ordering)**:
  - **Decision**: Hooks are identified by call order, not explicit keys (like `use_hook("my_id", ...)`) or variable names.
  - **Why**: Ergonomics. It allows concise API (`use_signal(|| 0)`) without checking for key collisions.
  - **Trade-off**: It creates the "Rules of Hooks". You cannot call hooks conditionally or in loops, because the order _must_ be constant between renders.

- **Dynamic storage (`Box<dyn Any>`)**:
  - **Decision**: Hooks are stored in a type-erased vector.
  - **Why**: Rust is statically typed, but a component's state is heterogeneous (a Signal, then a Coroutine, then a Memo). `dyn Any` is the only way to store them in a single list.
  - **Trade-off**: Requires a small runtime check (`downcast_ref`) on every access. In practice, this is negligible compared to DOM operations.

### Hooks: Rules of Hooks (Invariants)

To ensure that hooks map to the correct state every time, you must follow strict rules similar to React.

**Only Call Hooks at the Top Level**

Do not call Hooks inside loops, conditions, or nested functions. Instead, always use Hooks at the top level of your component function, before any early returns.

✅ Call them at the top level in the body of a component function.
✅ Call them at the top level in the body of a custom Hook.

**Prohibited Usage**

🔴 Do not call Hooks inside conditions or loops.
🔴 Do not call Hooks after a conditional return statement.
🔴 Do not call Hooks in event handlers.
🔴 Do not call Hooks inside closures passed to `use_memo` or `spawn`.

**Why?**
If the call order changes between renders, Dioxus will fail try to map the 2nd hook call to the 1st hook's state, causing a type mismatch panic or logic error.

**Examples of Mistakes**

```rust
fn BadComponent(props: Props) -> Element {
    // 🔴 Bad: inside a condition
    if props.enabled {
        let theme = use_context::<Theme>();
    }

    // 🔴 Bad: inside a loop
    for _ in 0..10 {
        let count = use_signal(|| 0);
    }

    // 🔴 Bad: after a conditional return
    if !props.ready {
        return rsx!("Loading...");
    }
    let _ = use_effect(move || {
        println!("Ready!");
    });

    // 🔴 Bad: inside an event handler
    rsx! {
        button {
            onclick: move |_| {
                let _count = use_signal(|| 0); // <--- Panic!
            }
        }
    }
}
```

### Hooks: Performance

- **Initialization**: Slow path. Allocates memory (Box) and runs the initializer. Happens only once per component instance.
- **Updates**: Fast path. Vector lookup + downcast. Very cheap.
- **Optimization**:
  - **Avoid huge hooks**: While the hook handle itself is usually small (Copy), implementing a custom hook that stores a massive struct in the hook list will increase memory usage. Prefer storing `Signal` or `Rc` handles.
  - **Hot Path**: The lookup is `O(1)`, but if you have thousands of hooks in a single component, you purely suffer from the linear scan of the "first render" allocation. In practice, code readability breaks before hook performance does.

## Props

In the component model, Props (short for Properties) are the arguments of a component function. Conceptually, they are just function arguments. They enable data to flow from a parent component down to a child component.

### Props: Ground Up Architecture

#### Level 1: The Struct (`packages/core/src/properties.rs`)

While you can write a component that takes no arguments, most components need data. In Dioxus, you define these arguments as a struct.

```rust
#[derive(Props, Clone, PartialEq)]
struct UserCardProps {
    name: String,
    age: i32,
    is_active: bool,
}
```

The `#[derive(Props)]` macro generates code that allows the `rsx!` macro to construct this struct by field name when you use the component.

#### Level 2: Ownership and Borrowing

Props in Dioxus operate on pass-by-value semantics. When a parent renders a child, it creates a new instance of the Props struct and moves it into the child component function.

- **Immutable Data Flow**: Because the props are moved into the function, they are owned by that specific render cycle. In Dioxus, props are typically immutable. You read from them to generate UI, but you do not modify them directly.

### Props: Best Practices and Performance

#### 1. Avoiding Unnecessary Copies with Signals

Standard props are cloned when passed down. For small data (integers, booleans), this is fine. For large data (like a list of 10,000 items), copying is slow.

Instead of passing the large data directly, pass a `Signal` (which is just a small pointer).

```rust
// Slow: Copies the vector
struct ListProps {
    items: Vec<String>
}

// Fast: Copies the pointer
struct ListProps {
    items: Signal<Vec<String>>
}
```

#### 2. Read-Only Signals (Principle of Least Privilege)

If you pass a mutable `Signal` to a child, that child has the power to write to it. Often, you want the child to _display_ data but not _change_ it.

You can convert a `Signal` into a `ReadOnlySignal`.

- **Safety**: The child gets a handle to the value, but the compiler prevents them from calling `.write()`.
- **Performance**: It is still just a pointer (`Copy`), so it is efficiently passed down without cloning the underlying data.
- **Reactivity**: The child component still automatically updates if the owner changes the value.
