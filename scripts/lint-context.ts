#!/usr/bin/env node

import { execSync } from "node:child_process";

/**
 * Context Lint Script
 *
 * Prevents accidental commits of external git repositories or unauthorized files
 * into the context directory.
 */

const ALLOWED_FILES: string[] = ["context/README.md"];

const ALLOWED_PREFIXES: string[] = [
  // Example: "context/allowed-sub-repo/"
];

// Patterns that are strictly forbidden (e.g., git internal files)
const FORBIDDEN_PATTERNS = [/\/\.git\//, /\/\.git$/];

function getTrackedFiles(cwd: string): string[] {
  try {
    const output = execSync("git ls-files context/", {
      cwd,
      encoding: "utf-8",
    });
    return output.split("\n").filter(Boolean);
  } catch (_error) {
    // If directory doesn't exist or git fails, return empty
    return [];
  }
}

async function run() {
  const cwd = process.cwd();
  const trackedFiles = getTrackedFiles(cwd);

  let errorCount = 0;

  for (const file of trackedFiles) {
    // Check forbidden patterns
    for (const pattern of FORBIDDEN_PATTERNS) {
      if (pattern.test(file)) {
        console.error(`❌ ERROR: Forbidden git-related file detected: ${file}`);
        errorCount++;
      }
    }

    // Check whitelist
    const isWhitelistedFile = ALLOWED_FILES.includes(file);
    const isWhitelistedPrefix = ALLOWED_PREFIXES.some((prefix) =>
      file.startsWith(prefix),
    );

    if (!isWhitelistedFile && !isWhitelistedPrefix) {
      console.error(`❌ ERROR: Untrusted file in context: ${file}`);
      console.error(
        `   If this file is intentional, add it to the whitelist in scripts/lint-context.ts`,
      );
      errorCount++;
    }
  }

  if (errorCount > 0) {
    console.error(`🚨 Found ${errorCount} lint error(s) in context/`);
    process.exit(1);
  }

  console.log(
    `✅ context/ lint check passed (${trackedFiles.length} files checked).`,
  );
  process.exit(0);
}

run();
