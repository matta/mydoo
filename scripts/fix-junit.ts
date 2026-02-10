import fs from "node:fs";
import path from "node:path";

/**
 * Patch JUnit XML content to include repo-relative 'file' attributes on test cases.
 * This is required for Trunk.io flaky test detection to correctly map results to source files.
 */
export function transformJunit(content: string, packageDir: string): string {
  const regex = /<testcase[^>]*>/g;
  return content.replace(regex, (tag) => {
    // Skip if 'file' attribute already exists
    if (tag.includes(' file="')) {
      return tag;
    }

    // Extract the classname which usually contains the package-relative path
    const classnameMatch = tag.match(/classname="([^"]*)"/);
    if (!classnameMatch || !classnameMatch[1]) {
      return tag;
    }

    const classname = classnameMatch[1];
    const repoRelativePath = path.join(packageDir, classname);

    // Insert file attribute before the closing `>` or `/>`
    // We use a regex to handle both standard `>` and self-closing `/>` tags,
    // and we consume any existing trailing whitespace to avoid double spaces.
    return tag.replace(/\s*(\/?)\/?>$/, ` file="${repoRelativePath}" $1>`);
  });
}

function main() {
  const junitPath = process.argv[2];
  const packageDir = process.argv[3]; // e.g., 'crates/tasklens-ui'

  if (!junitPath || !packageDir) {
    console.error("Usage: pnpm run fix-junit <junit.xml-path> <package-dir>");
    process.exit(1);
  }

  const absoluteJunitPath = path.resolve(junitPath);

  if (!fs.existsSync(absoluteJunitPath)) {
    console.error(`File not found: ${absoluteJunitPath}`);
    process.exit(1);
  }

  const content = fs.readFileSync(absoluteJunitPath, "utf8");
  const fixedContent = transformJunit(content, packageDir);

  if (fixedContent === content) {
    console.log(`No changes needed for ${junitPath}.`);
    return;
  }

  fs.writeFileSync(absoluteJunitPath, fixedContent);
  console.log(`Successfully fixed ${junitPath} for Trunk compatibility.`);
}

// Run main if this file is executed directly
if (process.argv[1]?.endsWith("fix-junit.ts") && !process.env.VITEST) {
  main();
}
