import importPlugin from "eslint-plugin-import-x";
import tseslint from "typescript-eslint";

export default tseslint.config(
  {
    ignores: [
      "**/dist/**",
      "**/build/**",
      "**/node_modules/**",
      "**/coverage/**",
      "**/.turbo/**",
      "**/.features-gen/**",
      "**/dev-dist/**",
      "**/.vercel/**",
      "**/pnpm-lock.yaml",
      "**/target/**",
      "packages/tasklens/specs/compliance/schemas/test-case.d.ts",
    ],
  },
  {
    files: ["**/*.ts", "**/*.tsx", "**/*.js", "**/*.jsx"],
    plugins: {
      "import-x": importPlugin,
    },
    languageOptions: {
      parser: tseslint.parser,
      ecmaVersion: "latest",
      sourceType: "module",
    },
    rules: {
      "import-x/no-namespace": "error",
      "import-x/first": "error",
      "import-x/newline-after-import": "error",
      "import-x/no-duplicates": "error",

      /**
       * Prevents "double casting" (e.g., const x = (y as unknown) as T).
       * This pattern is often used to bypass TypeScript's type-overlap safety checks.
       */
      "no-restricted-syntax": [
        "error",
        {
          selector:
            "TSAsExpression > TSAsExpression[typeAnnotation.type='TSUnknownKeyword']",
          message:
            "Do not double cast through 'unknown'. Use a type guard or refine your types instead.",
        },
        {
          selector:
            "TSAsExpression > TSAsExpression[typeAnnotation.type='TSAnyKeyword']",
          message:
            "Do not double cast through 'any'. Use a type guard or refine your types instead.",
        },
      ],
    },
  },
);
