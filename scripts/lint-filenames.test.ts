import { execSync } from "node:child_process";
import fs from "node:fs";
import { describe, expect, it, vi } from "vitest";
import { getTrackedFiles, loadConfig } from "./lint-filenames";

vi.mock("node:fs");
vi.mock("node:child_process");

describe("lint-filenames logic", () => {
  it("loadConfig should parse .ls-lint.yml", () => {
    vi.mocked(fs.existsSync).mockReturnValue(true);
    vi.mocked(fs.readFileSync).mockReturnValue(
      "ls:\n  .ts: kebab-case\nignore:\n  - node_modules",
    );

    const config = loadConfig();
    expect(config.ls[".ts"]).toBe("kebab-case");
    expect(config.ignore).toContain("node_modules");
  });

  it("loadConfig should throw if file not found", () => {
    vi.mocked(fs.existsSync).mockReturnValue(false);
    expect(() => loadConfig()).toThrow("Config file not found");
  });

  it("getTrackedFiles should return list of files", () => {
    vi.mocked(execSync).mockReturnValue(
      "file1.ts\nfile2.ts\n" as unknown as string,
    );

    const files = getTrackedFiles();
    expect(files).toEqual(["file1.ts", "file2.ts"]);
  });
});
