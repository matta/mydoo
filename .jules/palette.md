## 2024-05-24 - Dioxus Accessibility Attributes

**Learning:** Dioxus `rsx!` macro uses snake_case for HTML attributes, including ARIA attributes (e.g., `aria_label` instead of `aria-label`). This is a critical distinction from HTML/JSX.
**Action:** Always verify attribute names in Dioxus documentation or existing code when porting accessible patterns.

## 2024-05-25 - Dynamic ARIA Labels in Dioxus

**Learning:** In Dioxus RSX, `aria_label` (snake_case) accepts format strings directly (e.g., `aria_label: "Toggle completion for {title}"`), making it easy to create context-aware labels for lists without manual `format!` calls.
**Action:** Use inline format strings for dynamic accessible names to improve code readability and reduce verbosity.

## 2024-05-22 - Accessibility of Custom Interactive Elements

**Learning:** Interactive elements like `span`s used as buttons often lack keyboard accessibility in this codebase.
**Action:** When identifying `onclick` handlers on non-interactive elements, always add `role="button"`, `tabindex="0"`, `aria_label`, and `onkeydown` support for Enter/Space.
