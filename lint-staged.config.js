export default {
  'ROLLING_CONTEXT.md':
    "echo '❌ Error: ROLLING_CONTEXT.md is staged for commit.' && exit 1",
  '*.{js,ts,tsx,jsx,json,cjs,mjs}':
    'biome check --write --no-errors-on-unmatched',
  '*.{js,ts,tsx,jsx,cjs,mjs,config.js,config.ts}': 'eslint --fix',
  '*.{json,md,yaml,yml,js,mjs,cjs,ts,tsx,jsx,css,html}': 'prettier --write',
  // Run ls-lint on all files.
  // We use '*' to match all files. lint-staged handles chunking automatically for string commands.
  '*': 'ls-lint',

  // Run check-staged on the entire batch.
  // We use '**' (recursive wildcard) to match ALL files in the repo, ensuring this runs if any file is staged.
  // Using a distinct key from '*' allows us to separate this "run-once" logic (via function)
  // from the "command-string" logic above, ensuring check-filenames gets auto-chunking while this runs globally.
  '**': () => './scripts/check-staged.sh',
};
