export default {
  "ROLLING_CONTEXT.md":
    "echo '❌ Error: ROLLING_CONTEXT.md is staged for commit.' && exit 1",
  "*.{js,ts,tsx,jsx,json,cjs,mjs}":
    "biome check --write --no-errors-on-unmatched",
  "**/turbo.json": (files) =>
    files.map((f) => `pnpm tsx scripts/check-turbo-inputs.ts ${f}`),
  "*.{js,ts,tsx,jsx,cjs,mjs,config.js,config.ts}": "eslint --fix",
  "*.{json,md,yaml,yml,js,mjs,cjs,ts,tsx,jsx,css,html}": "prettier --write",
  // Run ls-lint on all files.
  // We use '*' to match all files. lint-staged handles chunking automatically for string commands.
  "*": "ls-lint",
};
