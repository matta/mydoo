/**
 * @file check-unused-catalog-entries.ts
 *
 * Checks for unused entries in the pnpm-workspace.yaml catalog.
 *
 * CONTEXT FOR NEWCOMERS:
 * - **pnpm**: A fast package manager (alternative to npm/yarn).
 * - **Workspaces**: A feature allowing multiple packages (apps/client, packages/tasklens)
 *   to exist in one git repository (monorepo).
 * - **Catalogs**: A generic pnpm feature where we define dependency versions in ONE place
 *   (pnpm-workspace.yaml) and reference them in individual package.json files using
 *   the `catalog:` protocol. This ensures all packages use the exact same version of "React"
 *   or "TypeScript" without manually syncing version numbers.
 * - **Zod**: A library for runtime data validation. We use it here to ensure that the
 *   files we read (package.json, pnpm-workspace.yaml) actually match the structure we expect,
 *   rather than just "trusting" (casting) them.
 *
 * GOAL OF THIS SCRIPT:
 * Identify definitions in the central catalog that are NOT actually used by any
 * package. This helps keep our configuration clean and avoids "ghost dependencies".
 *
 * NOTE: This script is a LAST RESORT.
 * We prefer standard tools like `syncpack` or `knip`, but they do not yet support
 * unused catalog detection natively.
 *
 * Usage: tsx scripts/check-unused-catalog-entries.ts
 */

// Node.js built-in modules
import fs from "node:fs"; // 'fs' = File System (reading/writing files)
import path from "node:path"; // Utilities for working with file and directory paths

// Third-party libraries
import { glob } from "glob"; // Matches files using patterns (like *.ts)
import yaml from "js-yaml"; // Parses YAML files into JavaScript objects
import { z } from "zod"; // Runtime schema validation

/**
 * Zod Schema: Defines the shape of 'pnpm-workspace.yaml'.
 *
 * Why Zod?
 * TypeScript interfaces vanish at runtime. Zod schemas exist at runtime.
 * This allows us to `.parse()` the actual data we read from disk and throw
 * a helpful error if the file structure is wrong (e.g., missing 'catalogs').
 */
const PnpmWorkspaceSchema = z.object({
  // catalogs: { [catalogName]: { [depName]: version } }
  catalogs: z.record(z.string(), z.record(z.string(), z.string())).optional(),
});

/**
 * Zod Schema: Defines the shape of a standard 'package.json'.
 *
 * We allow any record of strings for dependencies. We use `.optional()` because
 * a package might not have any dependencies at all.
 */
const PackageJsonSchema = z
  .object({
    dependencies: z.record(z.string(), z.string()).optional(),
    devDependencies: z.record(z.string(), z.string()).optional(),
    peerDependencies: z.record(z.string(), z.string()).optional(),
    optionalDependencies: z.record(z.string(), z.string()).optional(),
  })
  .passthrough();

const WORKSPACE_FILE = "pnpm-workspace.yaml";

async function main() {
  // process.cwd() returns the current directory where the script was run.
  // We assume this script is run from the root of the repo.
  const rootDir = process.cwd();
  const workspacePath = path.join(rootDir, WORKSPACE_FILE);

  // Safety check: Ensure the workspace file actually exists.
  if (!fs.existsSync(workspacePath)) {
    console.error(`Error: ${WORKSPACE_FILE} not found at ${workspacePath}`);
    // Exit code 1 tells the CI system (or terminal) that the script failed.
    process.exit(1);
  }

  // ---------------------------------------------------------------------------
  // STEP 1: Parse pnpm-workspace.yaml to find what IS defined.
  // ---------------------------------------------------------------------------

  const workspaceContent = fs.readFileSync(workspacePath, "utf8");
  const rawWorkspace = yaml.load(workspaceContent);

  // Validate the data against our Zod schema.
  // If the file is malformed, this line will throw a detailed error.
  const workspace = PnpmWorkspaceSchema.parse(rawWorkspace);

  if (!workspace.catalogs) {
    console.log("No catalogs found in pnpm-workspace.yaml.");
    return;
  }

  // We use a "Set" for efficient lookups.
  // A Set is a collection of unique values.
  // Structure:
  // {
  //   'default': Set('react', 'zod', 'typescript'),
  //   'react18': Set('react', 'react-dom')
  // }
  const definedCatalogEntries: Record<string, Set<string>> = {};

  for (const [catalogName, entries] of Object.entries(workspace.catalogs)) {
    definedCatalogEntries[catalogName] = new Set(Object.keys(entries));
  }

  // ---------------------------------------------------------------------------
  // STEP 2: Find all package.json files in the monorepo.
  // ---------------------------------------------------------------------------

  // 'glob' searches the file system.
  // '**' matches any number of subdirectories.
  // We explicitly ignore 'node_modules' (external code) and build artifacts.
  const packageJsonPaths = await glob("**/package.json", {
    ignore: ["**/node_modules/**", "**/dist/**", "**/coverage/**"],
    cwd: rootDir,
    absolute: true, // Return full system paths (/Users/me/project/...)
  });

  // ---------------------------------------------------------------------------
  // STEP 3: Scan every package.json to see what is USED.
  // ---------------------------------------------------------------------------

  // Create empty Sets to track usage for each catalog found in Step 1.
  const usedCatalogEntries: Record<string, Set<string>> = {};

  for (const catalogName of Object.keys(definedCatalogEntries)) {
    usedCatalogEntries[catalogName] = new Set();
  }

  for (const pkgPath of packageJsonPaths) {
    try {
      const content = fs.readFileSync(pkgPath, "utf8");
      const rawPkg = JSON.parse(content);

      let pkg: z.infer<typeof PackageJsonSchema>;
      try {
        pkg = PackageJsonSchema.parse(rawPkg);
      } catch (validationError) {
        if (validationError instanceof z.ZodError) {
          console.warn(
            `Warning: Skipping ${pkgPath} due to validation error:`,
            validationError.format(),
          );
        } else {
          console.warn(
            `Warning: Unexpected error validating ${pkgPath}`,
            validationError,
          );
        }
        continue;
      }

      // Combine all dependency types into one big object for easy iteration.
      // Spread syntax (...) merges properties from objects.
      const allDeps = {
        ...pkg.dependencies,
        ...pkg.devDependencies,
        ...pkg.peerDependencies,
        ...pkg.optionalDependencies,
      };

      // Loop through every dependency in this package.json
      for (const [depName, version] of Object.entries(allDeps)) {
        // We only care if the version string uses the 'catalog:' protocol.
        // Examples: "catalog:", "catalog:default", "catalog:special-catalog"
        if (
          version &&
          typeof version === "string" &&
          version.startsWith("catalog:")
        ) {
          const parts = version.split(":");
          // If split gives ["catalog", ""], it means implicit default catalog.
          // If split gives ["catalog", "my-cat"], it targets "my-cat".
          let targetCatalog = "default";

          if (parts.length > 1 && parts[1]) {
            targetCatalog = parts[1];
          }

          // If this catalog exists in our definitions...
          if (usedCatalogEntries[targetCatalog]) {
            // Mark this dependency as USED.
            // NOTE: In pnpm catalogs, the key in package.json (depName) MUST match
            // the key in the catalog for the mapping to work.
            usedCatalogEntries[targetCatalog].add(depName);
          }
        }
      }
    } catch (err) {
      // If a package.json is malformed (invalid JSON), log a warning but keep going.
      console.warn(`Warning: Could not parse JSON in ${pkgPath}`, err);
    }
  }

  // ---------------------------------------------------------------------------
  // STEP 4: Compare DEFINED vs USED and report results.
  // ---------------------------------------------------------------------------

  let hasUnused = false;
  console.log("Checking for unused pnpm catalog entries...");

  for (const [catalogName, definedSet] of Object.entries(
    definedCatalogEntries,
  )) {
    const usedSet = usedCatalogEntries[catalogName];

    // Safety check: Should always exist based on initialization above.
    if (!usedSet) continue;

    // Filter the 'defined' set to find items NOT present in the 'used' set.
    const unused = [...definedSet].filter((x) => !usedSet.has(x));

    if (unused.length > 0) {
      hasUnused = true;
      console.log(
        `\nCatalog '${catalogName}' has ${unused.length} unused entries:`,
      );
      for (const dep of unused) {
        console.log(`  - ${dep}`);
      }
    }
  }

  if (hasUnused) {
    console.log("\nFound unused catalog entries.");
    // Exit with error code 1 so CI pipelines fail.
    process.exit(1);
  } else {
    console.log("\nAll catalog entries are used!");
  }
}

// Execute main logic
main().catch((err) => {
  console.error(err);
  process.exit(1);
});
