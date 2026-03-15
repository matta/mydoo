## 2024-05-24 - Dioxus Accessibility Attributes

**Learning:** Dioxus `rsx!` macro uses snake_case for HTML attributes, including ARIA attributes (e.g., `aria_label` instead of `aria-label`). This is a critical distinction from HTML/JSX.
**Action:** Always verify attribute names in Dioxus documentation or existing code when porting accessible patterns.

## 2024-05-25 - Dynamic ARIA Labels in Dioxus

**Learning:** In Dioxus RSX, `aria_label` (snake_case) accepts format strings directly (e.g., `aria_label: "Toggle completion for {title}"`), making it easy to create context-aware labels for lists without manual `format!` calls.
**Action:** Use inline format strings for dynamic accessible names to improve code readability and reduce verbosity.
## 2024-03-15 - Add title attribute to icon-only buttons
**Learning:** While `aria_label` is great for screen readers, sighted mouse users and keyboard users also benefit from tooltips on icon-only buttons to understand their purpose.
**Action:** Always pair `aria_label` with a `title` attribute on icon-only buttons to ensure a native hover tooltip is displayed for all users.
