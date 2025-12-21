import js from "@eslint/js";
import tseslint from "typescript-eslint";
import globals from "globals";

/** Standard ignore patterns for all packages */
const sharedIgnores = {
  ignores: ["dist/", "dev-dist/", "node_modules/", "coverage/", "**/*.d.ts"],
};

/** Config file patterns that should bypass strict type-checking */
const configFilePatterns = [
  "**/*.config.{js,cjs,mjs,ts}",
  "**/*.{js,cjs,mjs}",
  "*.config.{js,cjs,mjs,ts}",
  "*.{js,cjs,mjs}",
];

/** Base shared configuration */
export const sharedConfig = [
  sharedIgnores,
  js.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  ...tseslint.configs.stylisticTypeChecked,
  {
    languageOptions: {
      globals: { ...globals.node, ...globals.browser },
      parserOptions: {
        projectService: {
          allowDefaultProject: configFilePatterns,
        },
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/no-floating-promises": "error",
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
    },
  },
  // Add config file overrides to disable type-checked rules globally.
  {
    ...tseslint.configs.disableTypeChecked,
    files: configFilePatterns,
  },
  {
    files: configFilePatterns,
    rules: {
      "@typescript-eslint/no-unsafe-assignment": "off",
      "@typescript-eslint/no-unsafe-call": "off",
      "@typescript-eslint/no-unsafe-member-access": "off",
    },
  },
];

/**
 * Configures the shared ESLint setup for a specific package directory.
 *
 * This function adapts the root `sharedConfig` for use in a specific package
 * by:
 * 1. Setting `tsconfigRootDir` to the package's directory so
 *    `typescript-eslint` can find the local `tsconfig.json`.
 *
 * @param {string} dirname - The absolute path to the package's root directory
 * (usually `import.meta.dirname`).
 * @returns {Array<Object>} A tailored ESLint configuration array for the
 * package.
 */
export function configureProject(dirname) {
  return sharedConfig.map((config) => {
    // Override tsconfigRootDir for this package
    if (config.languageOptions?.parserOptions) {
      return {
        ...config,
        languageOptions: {
          ...config.languageOptions,
          parserOptions: {
            ...config.languageOptions.parserOptions,
            tsconfigRootDir: dirname,
          },
        },
      };
    }
    return config;
  });
}

export default sharedConfig;
