import globals from "globals";
import pluginJs from "@eslint/js";
import tseslint from "typescript-eslint";

export default [
  {
    // Apply these settings to all JS/TS files
    files: ["**/*.{js,mjs,cjs,ts}"],
    languageOptions: {
      globals: globals.node,
      parserOptions: {
        // REQUIRED for Type-Checked rules: point to your tsconfig
        project: "./tsconfig.json",
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        tsconfigRootDir: /** @type {string} */ (import.meta.dirname),
      },
    },
  },

  // Base JS recommendations
  pluginJs.configs.recommended,

  // MAX STRICTNESS: Use "strictTypeChecked" instead of "recommended"
  // This enables rules that require type information (like checking for unhandled promises)
  ...tseslint.configs.strictTypeChecked,

  // OPTIONAL: Enforces consistent styling using type info
  ...tseslint.configs.stylisticTypeChecked,

  {
    rules: {
      // Re-enforce strictness that was previously turned off
      "@typescript-eslint/no-explicit-any": "error",

      // Force developers to handle all promises (prevents "fire and forget" bugs)
      "@typescript-eslint/no-floating-promises": "error",

      // Allow unused vars only if they start with _, otherwise Error
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
    },
  },
  {
    ignores: ["dist/", "node_modules/", "coverage/", "**/*.d.ts"],
  },
];
