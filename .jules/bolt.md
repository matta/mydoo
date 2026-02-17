## 2026-02-17 - FFI in Hot Loop
**Learning:** `js_sys::Date::now()` involves a WASM-to-JS boundary crossing (FFI) which can be expensive when called inside a tight loop (like rendering a list of hundreds of tasks).
**Action:** Lift `Date.now()` (and other JS interop calls) to the parent component and pass the value down as a prop, ensuring only one FFI call per render cycle.
