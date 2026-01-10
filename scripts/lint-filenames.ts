#!/usr/bin/env node

import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import yaml from "js-yaml";
import { minimatch } from "minimatch";
import { z } from "zod";

const ConfigSchema = z.object({
  ls: z.record(z.string(), z.string()),
  ignore: z.array(z.string()),
});

export type Config = z.infer<typeof ConfigSchema>;

const RULES: Record<string, RegExp> = {
  "kebab-case": /^[a-z0-9-.]+$/,
  snake_case: /^[a-z0-9_.]+$/,
  camelCase: /^[a-z][a-zA-Z0-9.]*$/,
  PascalCase: /^[A-Z][a-zA-Z0-9.]*$/,
  SCREAMING_SNAKE_CASE: /^[A-Z0-9_.]+$/,
};

export function loadConfig(cwd: string = process.cwd()): Config {
  const configPath = path.resolve(cwd, ".ls-lint.yml");
  if (!fs.existsSync(configPath)) {
    throw new Error(`Config file not found: ${configPath}`);
  }

  const content = fs.readFileSync(configPath, "utf-8");
  const rawConfig = yaml.load(content);
  return ConfigSchema.parse(rawConfig);
}

export function getTrackedFiles(cwd: string = process.cwd()): string[] {
  try {
    const output = execSync("git ls-files", { cwd, encoding: "utf-8" });
    return output.split("\n").filter(Boolean);
  } catch (error) {
    throw new Error(`Failed to get tracked files from git: ${error}`);
  }
}

export function isIgnored(filePath: string, ignorePatterns: string[]): boolean {
  return ignorePatterns.some((pattern) => {
    // Exact glob match
    if (minimatch(filePath, pattern, { dot: true })) return true;

    // Directory match: if the file starts with the pattern followed by a slash
    const cleanPattern = pattern.replace(/\/$/, "");
    if (filePath.startsWith(`${cleanPattern}/`)) return true;

    return false;
  });
}

export function validateName(name: string, rule: string): boolean {
  if (rule.startsWith("regex:")) {
    const pattern = rule.slice(6);
    return new RegExp(pattern).test(name);
  }

  const subRules = rule.split("|").map((r) => r.trim());
  if (subRules.length > 1) {
    return subRules.some((r) => validateName(name, r));
  }

  const regex = RULES[rule];
  if (!regex) {
    // Unknown rule or ignored (e.g. if config has weird stuff), treat as pass
    return true;
  }
  return regex.test(name);
}

export function checkFile(filePath: string, config: Config): string[] {
  const errors: string[] = [];
  const segments = filePath.split("/");
  const filename = segments.pop() || "";
  if (!filename) return errors;

  const dirSegments = segments;

  // Check directories
  const dirRule = config.ls[".dir"];
  if (dirRule) {
    for (const dir of dirSegments) {
      if (!validateName(dir, dirRule)) {
        errors.push(`Directory "${dir}" does not match rules: ${dirRule}`);
      }
    }
  }

  // Check filename
  let fileRule: string | null = null;
  let nameToCheck = filename;

  // Iterate config keys to find match
  for (const [pattern, rule] of Object.entries(config.ls)) {
    if (pattern === ".dir") continue;

    if (pattern.startsWith(".")) {
      // Extension match
      if (minimatch(filePath, `**/*${pattern}`, { dot: true })) {
        fileRule = rule;
        // Rules apply to the filename WITHOUT the matched extension
        nameToCheck = filename.slice(0, -pattern.length);
        break;
      }
    } else {
      // Glob match
      if (
        minimatch(filePath, pattern, { dot: true }) ||
        minimatch(filePath, `**/${pattern}`, { dot: true })
      ) {
        fileRule = rule;
        nameToCheck = filename;
        break;
      }
    }
  }

  if (fileRule && !validateName(nameToCheck, fileRule)) {
    errors.push(
      `File "${filename}" (stem: "${nameToCheck}") does not match rules: ${fileRule}`,
    );
  }

  return errors;
}

async function run() {
  try {
    const config = loadConfig();
    const files = getTrackedFiles();

    let ignoredCount = 0;
    let errorCount = 0;

    for (const file of files) {
      if (isIgnored(file, config.ignore)) {
        ignoredCount++;
        continue;
      }

      const errors = checkFile(file, config);
      if (errors.length > 0) {
        errorCount++;
        console.error(`ERROR: ${file}`);
        for (const err of errors) {
          console.error(`  - ${err}`);
        }
      }
    }

    console.log(
      `Checked ${files.length} files. Ignored ${ignoredCount}. Found ${errorCount} errors.`,
    );

    if (errorCount > 0) {
      process.exit(1);
    }
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exit(1);
  }
}

// Check if this script is being run directly
const argv1 = process.argv[1];
const nodePath = argv1 ? path.resolve(argv1) : "";
const scriptPath = fileURLToPath(import.meta.url);

if (nodePath === scriptPath || nodePath.endsWith("lint-filenames.ts")) {
  run();
}
