#!/usr/bin/env node

import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import yaml from "js-yaml";

export interface Config {
  ls: Record<string, string>;
  ignore: string[];
}

export function loadConfig(cwd: string = process.cwd()): Config {
  const configPath = path.resolve(cwd, ".ls-lint.yml");
  if (!fs.existsSync(configPath)) {
    throw new Error(`Config file not found: ${configPath}`);
  }

  const content = fs.readFileSync(configPath, "utf-8");
  return yaml.load(content) as Config;
}

export function getTrackedFiles(cwd: string = process.cwd()): string[] {
  try {
    const output = execSync("git ls-files", { cwd, encoding: "utf-8" });
    return output.split("\n").filter(Boolean);
  } catch (error) {
    throw new Error(`Failed to get tracked files from git: ${error}`);
  }
}

async function run() {
  try {
    const config = loadConfig();
    const files = getTrackedFiles();

    console.log(`Config loaded with ${Object.keys(config.ls).length} patterns`);
    console.log(`Checking ${files.length} tracked files`);
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
