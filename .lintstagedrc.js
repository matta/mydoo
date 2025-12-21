export default {
  '*': 'prettier --write --ignore-unknown',
  '**/*.{ts,tsx}': 'eslint --fix',
  // Check packages.json for consistency. Fails if invalid.
  '**/package.json': () => 'pnpm lint:deps',
};
