import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { next as Automerge } from "@automerge/automerge";
import { TunnelStateSchema } from "../persistence/schemas";

/**
 * Validates a single .automerge file.
 * Returns true if valid, false otherwise.
 */
function validateFile(filePath: string): boolean {
  // console.log(`Validating: ${filePath}`);
  try {
    const binary = readFileSync(filePath);
    const doc = Automerge.load(binary);
    const result = TunnelStateSchema.safeParse(doc);

    if (result.success) {
      // console.log(`✅ Success: ${filePath}`);
      return true;
    } else {
      console.error(`❌ Validation failed for ${filePath}:`);
      console.error(JSON.stringify(result.error.format(), null, 2));
      return false;
    }
  } catch (err) {
    console.error(`❌ Error reading or loading ${filePath}:`, err);
    return false;
  }
}

async function main() {
  const args = process.argv.slice(2);
  if (args.length === 0) {
    console.error("Usage: validate-compliance <file-or-directory>");
    process.exit(1);
  }

  const target = args[0];
  if (!target) {
    console.error("Usage: validate-compliance <file-or-directory>");
    process.exit(1);
  }
  const stats = statSync(target);

  let success = true;

  if (stats.isDirectory()) {
    const files = readdirSync(target)
      .filter((f: string) => f.endsWith(".automerge"))
      .map((f: string) => join(target, f));

    if (files.length === 0) {
      console.warn(`No .automerge files found in ${target}`);
      process.exit(0);
    }

    for (const file of files) {
      if (!validateFile(file)) {
        success = false;
      }
    }
  } else {
    success = validateFile(target);
  }

  process.exit(success ? 0 : 1);
}

main().catch((err) => {
  console.error("FATAL:", err);
  process.exit(1);
});
