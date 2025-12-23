import tseslint from 'typescript-eslint';

export default tseslint.config(
  {
    ignores: [
      '**/dist/**',
      '**/coverage/**',
      '**/node_modules/**',
      '**/*.config.*',
      '**/.turbo/**',
      '**/*.d.ts',
      '**/dev-dist/**',
    ],
  },
  {
    // STRICT TYPE CHECKING ONLY
    files: ['**/*.ts', '**/*.tsx'],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    extends: [...tseslint.configs.strictTypeChecked],
    rules: {
      // Turn off everything Biome handles
      '@typescript-eslint/consistent-type-definitions': 'off',
      '@typescript-eslint/array-type': 'off',
      '@typescript-eslint/no-unused-vars': 'off', // Biome handles this

      // Turn off rules that conflict with "Simplicity"
      '@typescript-eslint/prefer-nullish-coalescing': 'off',

      // KEEP THE CRITICAL SAFETY RULES
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/no-misused-promises': 'error',
      '@typescript-eslint/no-unsafe-assignment': 'error',
      '@typescript-eslint/no-unsafe-call': 'error',
      '@typescript-eslint/no-unsafe-member-access': 'error',
      '@typescript-eslint/no-unsafe-return': 'error',
    },
  },
  {
    // Allow JS config files to exist without type checking
    files: ['**/*.js', '**/*.mjs', '**/*.cjs'],
    extends: [tseslint.configs.disableTypeChecked],
  },
);
