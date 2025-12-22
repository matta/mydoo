import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import {defineConfig} from 'eslint/config';

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
]);
