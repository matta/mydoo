import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import {defineConfig} from 'eslint/config';
import tseslint from 'typescript-eslint';

import {configureProject} from '../../eslint.config.mjs';

export default defineConfig([
  ...configureProject(import.meta.dirname),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [reactHooks.configs.flat.recommended, reactRefresh.configs.vite],
  },
  {
    // Global restrictions for the Client App
    files: ['src/**/*.{ts,tsx}'],
    ignores: ['**/*.test.{ts,tsx}', 'src/test/**/*', 'src/tests/**/*'],
    rules: {
      'no-restricted-imports': [
        'error',
        {
          patterns: [
            {
              group: ['@automerge/*'],
              message:
                'Direct Automerge access is restricted. Use ViewModel hooks (@/viewmodel) or sanctioned infrastructure files.',
            },
            {
              group: ['@mydoo/tasklens/persistence/*'],
              message:
                'Deep imports from persistence layer are forbidden. Use the public @mydoo/tasklens API.',
            },
          ],
        },
      ],
    },
  },
  {
    // Exceptions: Infrastructure and specific Connector Hooks are allowed to use Automerge
    files: [
      'src/viewmodel/useDocument.ts',
      'src/hooks/RepoProvider.tsx',
      'src/lib/db.ts',
    ],
    rules: {
      // Re-define restricted imports to allow Automerge, but still ban deep persistence imports
      'no-restricted-imports': [
        'error',
        {
          patterns: [
            {
              group: ['@mydoo/tasklens/persistence/*'],
              message:
                'Deep imports from persistence layer are forbidden. Use the public @mydoo/tasklens API.',
            },
          ],
        },
      ],
    },
  },
  {
    // Test files - disable type-aware linting since they're excluded from main tsconfig
    ...tseslint.configs.disableTypeChecked,
    files: [
      '**/*.test.{ts,tsx}',
      'src/test/**/*.{ts,tsx}',
      'src/tests/**/*.{ts,tsx}',
    ],
  },
  {
    // Additional test file rules
    files: [
      '**/*.test.{ts,tsx}',
      'src/test/**/*.{ts,tsx}',
      'src/tests/**/*.{ts,tsx}',
    ],
    rules: {
      'react-refresh/only-export-components': 'off',
      // Test files can use Automerge directly
      'no-restricted-imports': 'off',
    },
  },
  {
    // E2E tests (Playwright) - uses CommonJS and doesn't need strict TS rules
    files: ['tests/e2e/**/*.{js,ts}'],
    rules: {
      '@typescript-eslint/no-require-imports': 'off',
      '@typescript-eslint/no-unsafe-assignment': 'off',
      '@typescript-eslint/no-unsafe-member-access': 'off',
      '@typescript-eslint/no-unsafe-call': 'off',
      '@typescript-eslint/no-unused-expressions': 'off',
      // Playwright tests are inherently complex
      'sonarjs/cognitive-complexity': 'off',
    },
  },
]);
