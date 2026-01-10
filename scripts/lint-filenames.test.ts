import { execSync } from "node:child_process";
import path from "node:path";
import { describe, expect, it } from "vitest";

describe("lint-filenames script", () => {
  it("should execute without error", () => {
    const scriptPath = path.resolve(__dirname, "lint-filenames.ts");
    // We use tsx to run it
    const output = execSync(`npx tsx ${scriptPath}`, { encoding: "utf-8" });
    expect(output).toContain("Lint filenames script started");
  });
});
