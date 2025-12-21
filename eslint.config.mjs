import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import globals from 'globals';
import sonarjs from 'eslint-plugin-sonarjs';
import perfectionist from 'eslint-plugin-perfectionist';

/** Standard ignore patterns for all packages */
const sharedIgnores = {
  ignores: ['dist/', 'dev-dist/', 'node_modules/', 'coverage/', '**/*.d.ts'],
};

/** Config file patterns that should bypass strict type-checking */
const configFilePatterns = [
  '*.config.{js,cjs,mjs,ts}',
  'eslint.config.{js,mjs}',
  'vite.config.{js,ts}',
  '.lintstagedrc.js',
  'apps/client/vite.config.ts',
];

/** Base shared configuration */
export const sharedConfig = [
  sharedIgnores,
  js.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  ...tseslint.configs.stylisticTypeChecked,
  {
    plugins: {
      '@typescript-eslint': tseslint.plugin,
      sonarjs,
      perfectionist,
    },
    languageOptions: {
      globals: {...globals.node, ...globals.browser},
      parserOptions: {
        projectService: {
          allowDefaultProject: configFilePatterns,
        },
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
        },
      ],

      // Google TypeScript Style (GTS) Rules (Inlined)
      // Rationale: The `gts` package currently only exports a legacy (.eslintrc) configuration.
      // Attempting to use it with `FlatCompat` caused version conflicts (typescript-eslint v5 vs v8).
      // We inline these standard rules here to maintain the Google Style without the legacy dependency complexity.
      'block-scoped-var': 'error',
      eqeqeq: 'error',
      'no-var': 'error',
      'prefer-const': 'error',
      'eol-last': 'error',
      'prefer-arrow-callback': 'error',
      'no-trailing-spaces': 'error',
      quotes: ['warn', 'single', {avoidEscape: true}],
      'no-restricted-properties': [
        'error',
        {object: 'describe', property: 'only'},
        {object: 'it', property: 'only'},
      ],

      // SonarJS Rules
      // TODO: Reduce Cognitive Complexity. The default for this plugin is 15.
      // We are at 40 to allow existing complex algorithm logic to pass.
      'sonarjs/cognitive-complexity': ['error', 40],

      // Perfectionist Rules
      'perfectionist/sort-imports': [
        'error',
        {
          type: 'natural',
          order: 'asc',
          groups: [
            ['builtin', 'external'],
            'internal',
            ['parent', 'sibling', 'index'],
            'side-effect',
            'style',
            'unknown',
          ],
        },
      ],
      'perfectionist/sort-named-imports': [
        'error',
        {
          type: 'natural',
          order: 'asc',
        },
      ],
      'perfectionist/sort-exports': [
        'error',
        {
          type: 'natural',
          order: 'asc',
        },
      ],
      'perfectionist/sort-named-exports': [
        'error',
        {
          type: 'natural',
          order: 'asc',
        },
      ],
      'perfectionist/sort-jsx-props': [
        'error',
        {
          type: 'natural',
          order: 'asc',
        },
      ],
      'perfectionist/sort-interfaces': [
        'error',
        {
          type: 'natural',
          order: 'asc',
          partitionByComment: true,
          partitionByNewLine: true,
        },
      ],
      'perfectionist/sort-object-types': [
        'error',
        {
          type: 'natural',
          order: 'asc',
          partitionByComment: true,
          partitionByNewLine: true,
        },
      ],
      'perfectionist/sort-enums': [
        'error',
        {
          type: 'natural',
          order: 'asc',
          partitionByComment: true,
          partitionByNewLine: true,
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
      '@typescript-eslint/no-unsafe-assignment': 'off',
      '@typescript-eslint/no-unsafe-call': 'off',
      '@typescript-eslint/no-unsafe-member-access': 'off',
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
  return sharedConfig.map(config => {
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
