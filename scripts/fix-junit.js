import fs from "node:fs";
import path from "node:path";

// This script adds a 'file' attribute to JUnit XML test cases for Trunk compatibility.
// It also ensures the file paths are relative to the repository root.

const junitPath = process.argv[2];
const packageDir = process.argv[3]; // e.g., 'crates/tasklens-ui'

if (!junitPath || !packageDir) {
  console.error("Usage: node fix-junit.js <junit.xml-path> <package-dir>");
  process.exit(1);
}

const absoluteJunitPath = path.resolve(junitPath);

if (!fs.existsSync(absoluteJunitPath)) {
  console.error(`File not found: ${absoluteJunitPath}`);
  process.exit(1);
}

const content = fs.readFileSync(absoluteJunitPath, "utf8");

// Simple regex-based replacement to add the 'file' attribute.
// We look for <testcase classname="path/to/test.ts" ...>
// and change it to <testcase classname="path/to/test.ts" file="package-dir/path/to/test.ts" ...>

const regex = /<testcase\s+name="([^"]*)"\s+classname="([^"]*)"/g;
const fixedContent = content.replace(regex, (match, name, classname) => {
  // Only add if file attribute isn't already there
  if (match.includes(' file="')) return match;

  const repoRelativePath = path.join(packageDir, classname);
  return `<testcase name="${name}" classname="${classname}" file="${repoRelativePath}"`;
});

fs.writeFileSync(absoluteJunitPath, fixedContent);
console.log(`Successfully fixed ${junitPath} for Trunk compatibility.`);
