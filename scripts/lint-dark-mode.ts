#!/usr/bin/env node

/**
 * Dark Mode Lint Script
 *
 * Checks Rust/Dioxus UI components for Tailwind CSS classes that may not
 * properly support dark mode. Specifically, it flags light-mode-only classes
 * (like `bg-white`, `text-gray-*`, etc.) that don't have corresponding
 * `dark:` variants.
 *
 * Usage:
 *   pnpm tsx scripts/lint-dark-mode.ts [--fix]
 *
 * The --fix flag will output suggested fixes but won't auto-apply them
 * (manual review is recommended for Tailwind class changes).
 */

import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

// Light mode classes and their required dark mode counterpart prefixes
// Format: [lightModePattern, darkModePrefix, description, suggestion]
const DARK_MODE_RULES: Array<{
  lightPattern: RegExp;
  darkPrefix: string;
  description: string;
  suggestion: string;
}> = [
  // Background colors (only match standalone bg-*, not hover:bg-* or other prefixed variants)
  {
    lightPattern: /(?<![:\w])bg-white\b/,
    darkPrefix: "dark:bg-",
    description: "bg-white without dark variant",
    suggestion: "Add dark:bg-stone-900 or dark:bg-stone-950",
  },
  {
    lightPattern: /(?<![:\w])bg-gray-50\b/,
    darkPrefix: "dark:bg-",
    description: "bg-gray-50 without dark variant",
    suggestion: "Add dark:bg-stone-800",
  },
  {
    lightPattern: /(?<![:\w])bg-gray-100\b/,
    darkPrefix: "dark:bg-",
    description: "bg-gray-100 without dark variant",
    suggestion: "Add dark:bg-stone-700",
  },
  {
    lightPattern: /(?<![:\w])bg-gray-200\b/,
    darkPrefix: "dark:bg-",
    description: "bg-gray-200 without dark variant",
    suggestion: "Add dark:bg-stone-700",
  },
  {
    lightPattern: /(?<![:\w])bg-gray-300\b/,
    darkPrefix: "dark:bg-",
    description: "bg-gray-300 without dark variant",
    suggestion: "Add dark:bg-stone-600",
  },

  // Text colors (gray shades that need dark variants)
  {
    lightPattern: /\btext-gray-[4-9]00\b/,
    darkPrefix: "dark:text-",
    description: "text-gray-* without dark variant",
    suggestion:
      "Add dark:text-stone-* variant (e.g., text-gray-500 -> dark:text-stone-400)",
  },
  {
    lightPattern: /\btext-gray-900\b/,
    darkPrefix: "dark:text-",
    description: "text-gray-900 without dark variant",
    suggestion: "Add dark:text-stone-100 or dark:text-white",
  },
  {
    lightPattern: /\btext-gray-800\b/,
    darkPrefix: "dark:text-",
    description: "text-gray-800 without dark variant",
    suggestion: "Add dark:text-stone-100 or dark:text-stone-200",
  },
  {
    lightPattern: /\btext-gray-700\b/,
    darkPrefix: "dark:text-",
    description: "text-gray-700 without dark variant",
    suggestion: "Add dark:text-stone-200 or dark:text-stone-300",
  },

  // Border colors
  {
    lightPattern: /\bborder-gray-[1-3]00\b/,
    darkPrefix: "dark:border-",
    description: "border-gray-* without dark variant",
    suggestion: "Add dark:border-stone-600 or dark:border-stone-700",
  },

  // Hover states (need dark variants too)
  {
    lightPattern: /\bhover:bg-gray-50\b/,
    darkPrefix: "dark:hover:bg-",
    description: "hover:bg-gray-50 without dark variant",
    suggestion: "Add dark:hover:bg-stone-800",
  },
  {
    lightPattern: /\bhover:bg-gray-100\b/,
    darkPrefix: "dark:hover:bg-",
    description: "hover:bg-gray-100 without dark variant",
    suggestion: "Add dark:hover:bg-stone-700",
  },
  {
    lightPattern: /\bhover:bg-gray-200\b/,
    darkPrefix: "dark:hover:bg-",
    description: "hover:bg-gray-200 without dark variant",
    suggestion: "Add dark:hover:bg-stone-700",
  },
  {
    lightPattern: /\bhover:text-gray-700\b/,
    darkPrefix: "dark:hover:text-",
    description: "hover:text-gray-700 without dark variant",
    suggestion: "Add dark:hover:text-stone-200",
  },
  {
    lightPattern: /\bhover:text-gray-800\b/,
    darkPrefix: "dark:hover:text-",
    description: "hover:text-gray-800 without dark variant",
    suggestion: "Add dark:hover:text-stone-100",
  },

  // Placeholder colors
  {
    lightPattern: /\bplaceholder-gray-[3-5]00\b/,
    darkPrefix: "dark:placeholder-",
    description: "placeholder-gray-* without dark variant",
    suggestion: "Add dark:placeholder-stone-500",
  },
];

interface Violation {
  file: string;
  line: number;
  column: number;
  content: string;
  description: string;
  suggestion: string;
}

function getUiRustFiles(cwd: string): string[] {
  try {
    // Get all .rs files in the tasklens-ui crate
    const output = execSync('git ls-files "crates/tasklens-ui/**/*.rs"', {
      cwd,
      encoding: "utf-8",
    });
    return output.split("\n").filter(Boolean);
  } catch (error) {
    throw new Error(`Failed to get tracked files from git: ${error}`);
  }
}

/**
 * Extracts all class string contents from a line of Rust/RSX code.
 * Handles patterns like: class: "...", class = "...", class: format_args!("...")
 */
function extractClassStrings(line: string): string[] {
  const results: string[] = [];

  // Match class: "..." or class = "..."
  const simplePattern = /class[:\s=]+["']([^"']+)["']/g;
  for (const match of line.matchAll(simplePattern)) {
    if (match[1]) {
      results.push(match[1]);
    }
  }

  // Match class: format_args!("...")
  const formatArgsPattern = /class:\s*format_args!\s*\(\s*["']([^"']+)["']/g;
  for (const match of line.matchAll(formatArgsPattern)) {
    if (match[1]) {
      results.push(match[1]);
    }
  }

  return results;
}

/**
 * Checks if a class string has a dark mode variant for a given light mode class.
 */
function hasDarkVariant(classString: string, darkPrefix: string): boolean {
  return classString.includes(darkPrefix);
}

function checkFile(filePath: string, cwd: string): Violation[] {
  const violations: Violation[] = [];
  const fullPath = path.join(cwd, filePath);

  if (!fs.existsSync(fullPath)) {
    return violations;
  }

  const content = fs.readFileSync(fullPath, "utf-8");
  const lines = content.split("\n");

  for (let lineNum = 0; lineNum < lines.length; lineNum++) {
    const line = lines[lineNum];
    if (line === undefined) {
      continue;
    }

    // Skip lines that don't look like they contain Tailwind classes
    if (!line.includes("class:") && !line.includes("class =")) {
      continue;
    }

    const classStrings = extractClassStrings(line);

    for (const classString of classStrings) {
      for (const {
        lightPattern,
        darkPrefix,
        description,
        suggestion,
      } of DARK_MODE_RULES) {
        // Check if the light mode class is present
        const match = lightPattern.exec(classString);
        if (match) {
          // Check if there's a corresponding dark variant anywhere in the same class string
          if (!hasDarkVariant(classString, darkPrefix)) {
            violations.push({
              file: filePath,
              line: lineNum + 1,
              column: match.index + 1,
              content: line.trim(),
              description,
              suggestion,
            });
          }
        }
      }
    }
  }

  return violations;
}

async function run() {
  const cwd = process.cwd();
  const files = getUiRustFiles(cwd);

  let totalViolations = 0;
  const allViolations: Violation[] = [];

  for (const file of files) {
    const violations = checkFile(file, cwd);
    if (violations.length > 0) {
      totalViolations += violations.length;
      allViolations.push(...violations);
    }
  }

  if (allViolations.length === 0) {
    process.exit(0);
  }

  console.log(`\n🌙 Dark Mode Lint Results\n`);
  console.log(
    `Found ${totalViolations} potential dark mode violation(s) in ${files.length} files:\n`,
  );

  // Group by file
  const byFile = new Map<string, Violation[]>();
  for (const v of allViolations) {
    const existing = byFile.get(v.file) || [];
    existing.push(v);
    byFile.set(v.file, existing);
  }

  for (const [file, violations] of byFile) {
    console.log(`📁 ${file}`);
    for (const v of violations) {
      console.log(`   Line ${v.line}: ${v.description}`);
      console.log(`   ${v.content}`);
      console.log(`   💡 ${v.suggestion}\n`);
    }
  }

  console.log(`\n📋 Summary: ${totalViolations} issue(s) found.`);
  console.log(`   Run with --help for more information.\n`);

  process.exit(1);
}

// Check if this script is being run directly
const argv1 = process.argv[1];
const nodePath = argv1 ? path.resolve(argv1) : "";
const scriptPath = fileURLToPath(import.meta.url);

if (nodePath === scriptPath || nodePath.endsWith("lint-dark-mode.ts")) {
  run();
}

export { checkFile, getUiRustFiles, DARK_MODE_RULES };
