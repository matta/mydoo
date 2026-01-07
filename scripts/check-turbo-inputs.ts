/**
 * @file check-turbo-inputs.ts
 * @description Lints turbo.json files to ensure "loose" recursive inputs (containing '**')
 * are explicitly excluded using the patterns defined in `turbo-exclusions.json`.
 * Also provides an audit mode to discover git-ignored directories that are missing from exclusions.
 */
import {execSync} from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import {glob} from 'glob';
import {minimatch} from 'minimatch';
import minimist from 'minimist';

import {z} from 'zod';

const EXCLUSIONS_FILE = 'turbo-exclusions.json';

const USAGE = `
Usage:
  pnpm tsx scripts/check-turbo-inputs.ts [options] [files...]

Options:
  --audit       Run in audit mode to find git-ignored directories missing from exclusions.
  --help, -h    Show this help message.

Arguments:
  [files...]    Specific turbo.json files to check. If omitted, checks all turbo.json files in the workspace.

Examples:
  pnpm tsx scripts/check-turbo-inputs.ts --audit
  pnpm tsx scripts/check-turbo-inputs.ts apps/web/turbo.json
`;

const ExclusionsSchema = z.array(z.string());

const TurboConfigSchema = z
  .object({
    tasks: z
      .record(
        z.string(),
        z
          .object({
            inputs: z.array(z.string()).optional(),
          })
          .passthrough(),
      )
      .optional(),
  })
  .passthrough();

type Exclusions = z.infer<typeof ExclusionsSchema>;
type TurboConfig = z.infer<typeof TurboConfigSchema>;

/** Parse and validate exclusions from the root directory. */
function parseExclusions(rootDir: string): Exclusions {
  const exclusionsPath = path.join(rootDir, EXCLUSIONS_FILE);

  if (!fs.existsSync(exclusionsPath)) {
    console.error(`Exclusions file not found at ${exclusionsPath}`);
    process.exit(1);
  }

  try {
    const raw = JSON.parse(fs.readFileSync(exclusionsPath, 'utf8'));
    return ExclusionsSchema.parse(raw);
  } catch (e) {
    console.error(`Failed to parse or validate ${exclusionsPath}:`, e);
    process.exit(1);
  }
}

/** Parse and validate a turbo.json file. */
function parseTurboConfig(filePath: string): TurboConfig | undefined {
  if (!fs.existsSync(filePath)) {
    console.warn(`File not found: ${filePath}`);
    return undefined;
  }

  try {
    const raw = JSON.parse(fs.readFileSync(filePath, 'utf8'));
    return TurboConfigSchema.parse(raw);
  } catch (e) {
    console.error(`Failed to parse or validate ${filePath}:`, e);
    return undefined;
  }
}

/** Check if inputs end with the required exclusions. */
function validateInputs(inputs: string[], exclusions: Exclusions): boolean {
  const lastN = inputs.slice(-exclusions.length);
  return JSON.stringify(lastN) === JSON.stringify(exclusions);
}

/** Report a violation for a specific task. */
function reportViolation(
  rootDir: string,
  filePath: string,
  taskName: string,
  exclusions: Exclusions,
  actualEnding: string[],
): void {
  console.error(
    `\u2716 Violation in ${path.relative(rootDir, filePath)} task "${taskName}":`,
  );
  console.error(
    `  Loose inputs (containing '**') must end with the exclusions from ${EXCLUSIONS_FILE}.`,
  );
  console.error(
    `  Expected ending:`,
    JSON.stringify(exclusions, null, 2)
      .split('\n')
      .map(l => `    ${l}`)
      .join('\n'),
  );
  console.error(
    `  Actual ending:  `,
    JSON.stringify(actualEnding, null, 2)
      .split('\n')
      .map(l => `    ${l}`)
      .join('\n'),
  );
}

/** Validate a single turbo.json file. Returns true if valid. */
function validateFile(
  filePath: string,
  exclusions: Exclusions,
  rootDir: string,
): boolean {
  const content = parseTurboConfig(filePath);
  if (!content) return false;

  const tasks = content.tasks || {};
  let isValid = true;

  for (const [taskName, taskConfig] of Object.entries(tasks)) {
    const inputs = taskConfig.inputs;
    if (!inputs) continue;

    const isLoose = inputs.some((input: string) => input.includes('**'));
    if (isLoose && !validateInputs(inputs, exclusions)) {
      reportViolation(
        rootDir,
        filePath,
        taskName,
        exclusions,
        inputs.slice(-exclusions.length),
      );
      isValid = false;
    }
  }

  return isValid;
}

/**
 * Audit mode: Find all git-ignored directories and ensure they are covered
 * by at least one pattern in turbo-exclusions.json.
 */
async function auditExclusions(
  rootDir: string,
  exclusions: Exclusions,
): Promise<boolean> {
  console.log('Running audit...');

  // 1. Convert exclusions to positive globs (remove leading '!')
  const exclusionGlobs = exclusions.map(e => e.replace(/^!/, ''));

  // 2. Find all directories contents that are ignored by git
  let ignoredItems: string[] = [];
  try {
    const output = execSync(
      'git ls-files --others --ignored --exclude-standard --directory',
      {
        cwd: rootDir,
        encoding: 'utf8',
      },
    );
    ignoredItems = output.split('\n').filter(Boolean);
  } catch (e) {
    console.error('Failed to run git ls-files:', e);
    return false;
  }

  // 3. Filter for directories only (ending in /)
  const ignoredDirs = ignoredItems.filter(item => item.endsWith('/'));

  let hasError = false;

  // 4. For each ignored dir, check if it matches any exclusion glob
  for (const ignoredDir of ignoredDirs) {
    // minimatch expects forward slashes, git output is already using them typically
    // but good to be safe. Also git output has trailing slash, which we want.
    const normalizedDir = ignoredDir.split(path.sep).join('/');

    const isCovered = exclusionGlobs.some(pattern =>
      minimatch(normalizedDir, pattern),
    );

    if (!isCovered) {
      console.error(
        `\u2716 Missing exclusion for ignored directory: ${normalizedDir}`,
      );
      hasError = true;
    }
  }

  if (hasError) {
    console.error(
      `\nERROR: Found git-ignored directories that are NOT covered by ${EXCLUSIONS_FILE}.`,
    );
    console.error(
      '       These must be added to prevent accidental inclusion in Turbo inputs.',
    );
    return false;
  }

  console.log(
    `\u2714 Audit complete. All ${ignoredDirs.length} ignored directories are covered by exclusions.`,
  );
  return true;
}

async function main() {
  const rootDir = process.cwd();

  const argv = minimist(process.argv.slice(2), {
    boolean: ['audit', 'help'],
    alias: {h: 'help'},
  });

  if (argv.help) {
    console.log(USAGE);
    process.exit(0);
  }

  // Parse exclusions AFTER checking for help, so help runs even if exclusions are missing
  const exclusions = parseExclusions(rootDir);

  // 1. Audit Mode
  if (argv.audit) {
    const success = await auditExclusions(rootDir, exclusions);
    if (!success) process.exit(1);
    process.exit(0);
  }

  // 2. File Check Mode
  const files = argv._; // Positional arguments
  let targetFiles: string[] = [];

  if (files.length > 0) {
    targetFiles = files.filter((f: string) => f.endsWith('turbo.json'));
    if (targetFiles.length === 0) {
      process.exit(0); // Files provided but none matched turbo.json
    }
  } else {
    targetFiles = await glob('**/turbo.json', {
      ignore: '**/node_modules/**',
      cwd: rootDir,
      absolute: true,
    });
  }

  let hasError = false;
  for (const file of targetFiles) {
    const filePath = path.isAbsolute(file) ? file : path.resolve(rootDir, file);
    if (!validateFile(filePath, exclusions, rootDir)) {
      hasError = true;
    }
  }

  if (hasError) {
    process.exit(1);
  }

  console.log(
    `\u2714 ${targetFiles.length} turbo.json file(s) checked. All inputs valid.`,
  );
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
