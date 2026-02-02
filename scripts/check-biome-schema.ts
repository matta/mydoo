/**
 * Checks that the biome.json $schema version matches the installed biome CLI version.
 * Exits with code 1 if there's a mismatch, making CI fail when biome is upgraded
 * but the config isn't updated.
 */

import { execSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join, relative } from "node:path";

interface PnpmListEntry {
  devDependencies?: {
    "@biomejs/biome"?: {
      version: string;
    };
  };
  dependencies?: {
    "@biomejs/biome"?: {
      version: string;
    };
  };
}

function getInstalledBiomeVersion(rootDir: string): string {
  const output = execSync("pnpm list @biomejs/biome --json", {
    cwd: rootDir,
    encoding: "utf-8",
  });
  const parsed: PnpmListEntry[] = JSON.parse(output);
  const entry = parsed[0];
  const version =
    entry?.devDependencies?.["@biomejs/biome"]?.version ??
    entry?.dependencies?.["@biomejs/biome"]?.version;
  if (!version) {
    throw new Error("Could not find @biomejs/biome in pnpm list output");
  }
  return version;
}

function getSchemaVersion(biomeConfigPath: string): string | null {
  const config = JSON.parse(readFileSync(biomeConfigPath, "utf-8"));
  const schema: unknown = config.$schema;
  if (typeof schema !== "string") {
    return null;
  }
  // Extract version from URL like: https://biomejs.dev/schemas/2.3.12/schema.json
  const match = schema.match(/\/schemas\/([^/]+)\/schema\.json$/);
  return match?.[1] ?? null;
}

function main(): void {
  const rootDir = join(import.meta.dirname, "..");
  const biomeConfigPath = join(rootDir, "biome.json");

  const installedVersion = getInstalledBiomeVersion(rootDir);
  const schemaVersion = getSchemaVersion(biomeConfigPath);

  const scriptName = "check-biome-schema.ts";
  const relativeConfigPath = relative(rootDir, biomeConfigPath);

  if (schemaVersion === null) {
    console.error(
      `ERROR [${scriptName}]: Could not extract version from ` +
        `${relativeConfigPath} $schema`,
    );
    process.exit(1);
  }

  if (installedVersion !== schemaVersion) {
    console.error(
      `ERROR [${scriptName}]: ${relativeConfigPath} schema version ` +
        `${schemaVersion} does not match installed CLI version ` +
        `${installedVersion}. Run 'pnpm biome migrate' to fix.`,
    );
    process.exit(1);
  }
}

main();
