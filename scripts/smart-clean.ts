import { spawnSync } from "node:child_process";
import { rmSync } from "node:fs";
import { createInterface } from "node:readline";

// --- Configuration ---

// These patterns (checked against the relative path) are considered "High Confidence"
// safe to delete without deep scrutiny.
const HIGH_CONFIDENCE_PATTERNS = [
  /node_modules/,
  /dist\//,
  /\.turbo\//,
  /coverage\//,
  /playwright-report\//,
  /test-results\//,
  /\.tsbuildinfo/,
  /\.husky\/_/,
  /src\/generated/,
];

// Arguments passed to git clean to find ignored files.
// Matches scripts/aggressive-git-clean.sh
const GIT_CLEAN_ARGS = [
  "clean",
  "-fdX", // force, directories, ignored only
  "-n", // dry run (critical!)
  "-e",
  "!.proto",
  "-e",
  "!.env",
  "-e",
  "!*.local",
  "-e",
  "!.vscode",
  "-e",
  "!.gemini",
  "-e",
  "!.specify",
];

// --- Helpers ---

const ask = (query: string): Promise<boolean> => {
  const rl = createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  return new Promise((resolve) => {
    rl.question(`${query} [y/N] `, (answer) => {
      rl.close();
      resolve(answer.toLowerCase() === "y" || answer.toLowerCase() === "yes");
    });
  });
};

// --- Main ---

async function main() {
  console.log("Analyzing repository for ignored files...");

  // 1. Get the list of potential deletions from git
  const result = spawnSync("git", GIT_CLEAN_ARGS, { encoding: "utf-8" });

  if (result.error) {
    console.error("Failed to run git clean:", result.error);
    process.exit(1);
  }

  const rawOutput = result.stdout.trim();

  if (!rawOutput) {
    console.log("Repository is already clean.");
    return;
  }

  // Parse output: "Would remove path/to/file"
  const paths = rawOutput
    .split("\n")
    .map((line) => line.replace("Would remove ", "").trim())
    .filter((p) => p.length > 0);

  // 2. Categorize
  const highConfidencePaths: string[] = [];
  const otherPaths: string[] = [];

  for (const p of paths) {
    if (HIGH_CONFIDENCE_PATTERNS.some((regex) => regex.test(p))) {
      highConfidencePaths.push(p);
    } else {
      otherPaths.push(p);
    }
  }

  // 3. Process High Confidence
  if (highConfidencePaths.length > 0) {
    console.log(
      `\nFound ${highConfidencePaths.length} 'High Confidence' items (node_modules, dist, logs, etc.).`,
    );
    console.log(
      `Examples: ${highConfidencePaths.slice(0, 3).join(", ")}${highConfidencePaths.length > 3 ? "..." : ""}`,
    );

    const shouldDelete = await ask("Delete all high confidence items?");
    if (shouldDelete) {
      console.log("Deleting...");
      for (const p of highConfidencePaths) {
        try {
          rmSync(p, { recursive: true, force: true });
        } catch (e) {
          console.error(`Failed to delete ${p}:`, e);
        }
      }
      console.log("✅ High confidence items deleted.");
    } else {
      console.log("Skipped.");
    }
  } else {
    console.log("\nNo high confidence items found.");
  }

  // 4. Process Others
  if (otherPaths.length > 0) {
    console.log(`\nFound ${otherPaths.length} 'Other' items.`);

    if (otherPaths.length < 20) {
      console.log("Items:");
      for (const p of otherPaths) {
        console.log(` - ${p}`);
      }
    } else {
      console.log("First 10 items:");
      for (const p of otherPaths.slice(0, 10)) {
        console.log(` - ${p}`);
      }
      console.log(`... and ${otherPaths.length - 10} more.`);
    }

    const shouldDelete = await ask("Delete these items?");
    if (shouldDelete) {
      console.log("Deleting...");
      for (const p of otherPaths) {
        try {
          rmSync(p, { recursive: true, force: true });
        } catch (e) {
          console.error(`Failed to delete ${p}:`, e);
        }
      }
      console.log("✅ Other items deleted.");
    } else {
      console.log("Skipped.");
    }
  } else {
    console.log("\nNo other ignored items found.");
  }

  console.log("\nClean check complete.");
}

main();
