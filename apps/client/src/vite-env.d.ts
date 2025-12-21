/**
 * Vite Type Definitions
 *
 * This file configures TypeScript to support the non-standard import patterns used
 * in Vite projects. It uses two special TypeScript constructs to "teach" the
 * compiler about files that aren't JavaScript.
 *
 * 1. **Triple-Slash Directives** (`/// <reference types="..." />`)
 *    These pull in Vite's official type definitions from `node_modules`.
 *    - Enables: `import logo from "./logo.svg"` (treating images as URL strings).
 *    - Enables: `import.meta.env` (accessing build environment variables).
 *
 * 2. **Wildcard Module Declarations** (`declare module "*.css"`)
 *    This tells TypeScript: "Any import path ending in .css is a valid module."
 *    - Enables: `import "./styles.css"` in your components.
 *
 * **Why import assets in JavaScript?**
 * Importing non-code assets (CSS, SVG) directly into your components allows the
 * bundler (Vite) to track them as explicit dependencies.
 * - **Bundling**: Styles are only included if the component importing them is used.
 * - **Hashing**: Assets are automatically hashed for cache busting (e.g., `logo.a1b2.svg`).
 * - **Co-location**: You can keep `Button.tsx` and `Button.css` together.
 *
 * **Reference**: https://vite.dev/guide/features.html#client-types
 */

/// <reference types="vite/client" />
/// <reference types="vite-plugin-pwa/client" />

declare module "*.css";
