## 2024-05-24 - Dioxus Accessibility Attributes

**Learning:** Dioxus `rsx!` macro uses snake_case for HTML attributes, including ARIA attributes (e.g., `aria_label` instead of `aria-label`). This is a critical distinction from HTML/JSX.
**Action:** Always verify attribute names in Dioxus documentation or existing code when porting accessible patterns.
