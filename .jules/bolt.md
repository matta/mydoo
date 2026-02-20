## 2026-02-17 - FFI in Hot Loop

**Learning:** `js_sys::Date::now()` involves a WASM-to-JS boundary crossing (FFI) which can be expensive when called inside a tight loop (like rendering a list of hundreds of tasks).
**Action:** Lift `Date.now()` (and other JS interop calls) to the parent component and pass the value down as a prop, ensuring only one FFI call per render cycle.

## 2024-05-23 - Avoid Deep Clones in Dioxus Props

**Learning:** Passing large objects (like `String` or `Vec`) by value to Dioxus component props causes deep cloning on every render, even if the data hasn't changed. Wrapping them in `Rc<T>` allows cheap cloning of the reference.
**Action:** When passing large data to child components in a list, prefer `Rc<T>` over `T` to minimize allocation overhead.
