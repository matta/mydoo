import {configureProject} from '../../eslint.config.mjs';

export default [
  ...configureProject(import.meta.dirname),
  {
    files: ['src/domain/**/*'],
    rules: {
      'no-restricted-imports': [
        'error',
        {
          patterns: [
            {
              group: ['@automerge/*', 'react'],
              message:
                'Domain logic must be pure and independent of persistence and UI frameworks.',
            },
          ],
        },
      ],
    },
  },
];
