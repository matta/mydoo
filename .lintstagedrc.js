export default {
  '*': 'prettier --write --ignore-unknown',
  '**/*.{js,mjs,ts,tsx}': 'eslint --fix',
  // Check packages.json for consistency. Fails if invalid.
  '**/package.json': () => 'pnpm lint:deps',
};
