/**
 * Checks that the biome.json $schema version matches the installed biome CLI version.
 * Exits with code 1 if there's a mismatch, making CI fail when biome is upgraded
 * but the config isn't updated.
 */

import { execSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";

interface BiomeCheckResult {
  summary: {
    infos: number;
  };
  diagnostics: Array<{
    category: string;
    description: string;
  }>;
}

function main(): void {
  const rootDir = join(import.meta.dirname, "..");
  const biomeConfigPath = join(rootDir, "biome.json");

  // Run biome check with JSON reporter to get structured output
  let output: string;
  try {
    output = execSync("pnpm biome check . --reporter=json", {
      cwd: rootDir,
      encoding: "utf-8",
      stdio: ["pipe", "pipe", "pipe"],
    });
  } catch (error) {
    // biome check may exit non-zero for other reasons, but we still get output
    const execError = error as { stdout?: string; stderr?: string };
    output = execError.stdout ?? "";
    if (!output) {
      console.error("Failed to run biome check:", execError.stderr);
      process.exit(1);
    }
  }

  // Parse JSON output (skip the unstable warning line if present)
  const jsonLine = output
    .split("\n")
    .find((line) => line.trim().startsWith("{"));
  if (!jsonLine) {
    console.error("Could not parse biome JSON output");
    process.exit(1);
  }

  const result: BiomeCheckResult = JSON.parse(jsonLine);

  // Look for schema version mismatch diagnostic
  const schemaMismatch = result.diagnostics.find(
    (d) =>
      d.category === "deserialize" &&
      d.description.includes("configuration schema version does not match"),
  );

  if (schemaMismatch) {
    // Read the config to show current value
    const config = JSON.parse(readFileSync(biomeConfigPath, "utf-8"));
    const currentSchema = config.$schema ?? "(not set)";

    // Get installed version
    const versionOutput = execSync("pnpm biome --version", {
      encoding: "utf-8",
    }).trim();
    const installedVersion = versionOutput.replace("Version: ", "");

    console.error("ERROR: Biome schema version mismatch detected!");
    console.error(`  Config $schema: ${currentSchema}`);
    console.error(`  Installed CLI:  ${installedVersion}`);
    console.error("");
    console.error("Run 'pnpm biome migrate' to update the configuration.");
    process.exit(1);
  }

  console.log("✓ Biome schema version matches CLI version");
}

main();
