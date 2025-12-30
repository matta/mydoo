import importPlugin from 'eslint-plugin-import-x';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  {
    ignores: [
      '**/dist/**',
      '**/build/**',
      '**/node_modules/**',
      '**/coverage/**',
      '**/.turbo/**',
      '**/.features-gen/**',
      '**/dev-dist/**',
      '**/.vercel/**',
      '**/pnpm-lock.yaml',
    ],
  },
  {
    files: ['**/*.ts', '**/*.tsx', '**/*.js', '**/*.jsx'],
    plugins: {
      'import-x': importPlugin,
    },
    languageOptions: {
      parser: tseslint.parser,
      ecmaVersion: 'latest',
      sourceType: 'module',
    },
    rules: {
      'import-x/no-namespace': 'warn',
    },
  },
);
