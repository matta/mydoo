# Building a Type-Safe Design System with Dioxus & Tailwind

This guide summarizes the architecture for creating a consistent, refactor-safe design system using `tailwind-fuse` in Dioxus. This approach relies on separating **Design Tokens** (values) from **Component Logic** (usage).

## 1. The Core Philosophy

- **Type-Safe Consumption**: The safety lies in the _API surface_. Developers cannot pick a color or size that doesn't exist in the Rust Enums (`Intent`, `Size`).
- **String-Based Implementation**: The mapping to CSS classes relies on strings. This is _not_ validated by the Rust compiler, but is validated by the VS Code Tailwind CSS extension.
- **Tailwind Config (`.js`)**: The source of truth for values.
- **Rust / `tailwind-fuse`**: The bridge that restricts usage to valid combinations.

---

## 2. Layer 1: The Tokens (Tailwind Config)

> **Visual Foundation:** Tailwind includes a comprehensive baked-in design system ("Layer 0") with a tuned palette (slate, zinc, etc.), spacing scale, and typography.
> **Customization:** While entirely optional, we formally adopt a **Semantic Layer** to decouple our codebase from raw design values (colors, spacing, radii, etc.).

**Responsibility:** Define semantic names, not raw values.
**Rule:** Avoid "Magic Values" in your code. Never use `text-white` or `bg-blue-600` in Rust if it represents a semantic concept.

- **Tailwind defaults** provide the _vocabulary_ (e.g., `blue-600`).
- **Layer 1** provides the _meaning_ (e.g., `primary`).

### The "On-Color" Concept (Foregrounds)

We use the suffix `-foreground` (e.g., `primary-foreground`) to define the text color that sits _on top of_ a background.

- **Why?** It strictly couples content to its context. If you change `primary` from dark blue to bright yellow, you update `foreground` to black in _one place_, and every primary button in the app updates its contrast automatically.
- **Pattern:** `bg-primary text-primary-foreground`.

```javascript
// tailwind.config.js
const colors = require("tailwindcss/colors");

module.exports = {
  theme: {
    extend: {
      colors: {
        // 1. PRIMARY (Main Actions)
        // Mapped to Blue: The internet standard for "Action".
        primary: {
          ...colors.blue,
          DEFAULT: colors.blue[600], // 'bg-primary'
          foreground: colors.white, // 'text-primary-foreground'
        },

        // 2. SECONDARY (Neutral/Subtle)
        // Mapped to Slate: A modern, cool grey with a slight blue tint.
        secondary: {
          ...colors.slate,
          DEFAULT: colors.slate[200],
          foreground: colors.slate[900],
        },

        // 3. SUCCESS (Positive States)
        // Mapped to Emerald: Distinct from Blue, feels "fresh" and correct.
        success: {
          ...colors.emerald,
          DEFAULT: colors.emerald[600],
          foreground: colors.white,
        },

        // 4. WARNING (Cautionary States)
        // Mapped to Amber: Better than Yellow, which is often hard to read (low contrast).
        warning: {
          ...colors.amber,
          DEFAULT: colors.amber[500],
          foreground: colors.amber[950], // Dark text for contrast against amber
        },

        // 5. DANGER (Destructive Actions)
        // Mapped to Red: Universal signal for errors and deletion.
        danger: {
          ...colors.red,
          DEFAULT: colors.red[600],
          foreground: colors.white,
        },

        // 6. INFO (Helpful Context)
        // Mapped to Sky: distinct from Primary Blue, lighter and friendlier.
        info: {
          ...colors.sky,
          DEFAULT: colors.sky[500],
          foreground: colors.sky[950],
        },
      },
      // Standardize your structural tokens
      borderRadius: {
        standard: "0.375rem", // e.g. 'rounded-standard' instead of 'rounded-md'
      },
    },
  },
};
```

---

## 3. Layer 2: The Logic (Rust & `tailwind-fuse`)

**Responsibility:** Enforce the design language at compile time.
**Rule:** Use `TwVariant` for choices (Intents, Sizes) and `TwClass` to combine them. Map the semantic tokens from Layer 1 to these variants.

```rust
use tailwind_fuse::*;

// 1. Define Intents (Semantic Colors)
#[derive(TwVariant, Clone, Copy, PartialEq)]
pub enum Intent {
    #[tw(default, class = "bg-primary hover:bg-primary-700 text-primary-foreground")]
    Primary,

    #[tw(class = "bg-secondary hover:bg-secondary-300 text-secondary-foreground")]
    Secondary,

    #[tw(class = "bg-danger text-danger-foreground")]
    Danger,
}

// 2. Define Sizes (Dimensions)
#[derive(TwVariant, Clone, Copy, PartialEq)]
pub enum Size {
    #[tw(default, class = "h-10 px-4 py-2 text-base")]
    Medium,

    #[tw(class = "h-8 px-3 text-sm")]
    Small,
}

// 3. Define the Component Class Structure
#[derive(TwClass)]
// Base styles common to ALL buttons (layout, focus rings, transitions)
// Note usage of 'rounded-standard' from config
#[tw(class = "inline-flex items-center justify-center rounded-standard font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2")]
pub struct ButtonClass {
    pub intent: Intent,
    pub size: Size,
}
```

---

## 4. Layer 3: The Component (Dioxus)

**Responsibility:** expose a clean, typed API.
**Rule:** Allow an "escape hatch" (`class` prop) but use `merge_with` to handle conflicts intelligently.

```rust
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    // Users must pick from the defined Design System
    #[props(default)]
    intent: Intent,

    #[props(default)]
    size: Size,

    // The "Escape Hatch": allows one-off overrides like "m-4"
    #[props(default)]
    class: String,

    children: Element,
}

pub fn Button(props: ButtonProps) -> Element {
    // 1. Generate the base class from Props
    let base_class = ButtonClass {
        intent: props.intent,
        size: props.size,
    };

    // 2. Merge with any overrides (e.g., if user passes 'w-full')
    // 'tailwind-fuse' ensures this merge is clean and conflict-free.
    let final_class = base_class.merge_with(&props.class);

    rsx! {
        button {
            class: "{final_class}",
            {props.children}
        }
    }
}
```

---

## Summary of Responsibilities

| Location                 | Responsibility                                          | Example                     |
| :----------------------- | :------------------------------------------------------ | :-------------------------- |
| **`tailwind.config.js`** | **Values (Tokens)**. Defines what colors/sizes _exist_. | `colors: { primary: ... }`  |
| **`Rust Enum`**          | **Vocabulary**. Defines what choices a developer _has_. | `enum Intent { Primary }`   |
| **`Rust Struct`**        | **Composition**. Maps choices to CSS classes.           | `#[tw(class="bg-primary")]` |
| **`Dioxus Props`**       | **API**. Exposes the system to the app.                 | `intent: Intent`            |

---

## Appendix: Multi-Crate Workspace Setup (DaisyUI / Shared CSS)

For workspaces sharing UI across mobile, web, and desktop, you can centralize your Tailwind configuration using CSS imports. This ensures standard configuration handling with the Dioxus CLI (`dx`).

### 1. Shared Configuration (`packages/ui/shared.css`)

```css
@import "tailwindcss";

@source "./src/**/*.{rs,html,css}";

@source not "./shared.css";
```

### 2. Consumer Configuration (e.g., `packages/mobile/tailwind.css`)

```css
@import "../ui/shared.css";

@source "./src/**/*.{rs,html,css}";

@source not "./tailwindcss";
```

This approach allows you to import **DaisyUI** or define custom classes in `shared.css` and have them available in all downstream apps.
