import { execSync } from "node:child_process";
import fs from "node:fs";
import { describe, expect, it, vi } from "vitest";
import {
  checkFile,
  getTrackedFiles,
  isIgnored,
  loadConfig,
  validateName,
} from "./lint-filenames";

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
    // Mock returning a string. ReturnType of execSync includes string when encoding is provided.
    vi.mocked(execSync).mockReturnValue("file1.ts\nfile2.ts\n");

    const files = getTrackedFiles();
    expect(files).toEqual(["file1.ts", "file2.ts"]);
  });

  it("isIgnored should match patterns correctly", () => {
    const ignore = ["node_modules", "**/*.d.ts", "dist/**"];
    expect(isIgnored("node_modules/foo.js", ignore)).toBe(true);
    expect(isIgnored("src/foo.d.ts", ignore)).toBe(true);
    expect(isIgnored("dist/bundle.js", ignore)).toBe(true);
    expect(isIgnored("src/foo.ts", ignore)).toBe(false);
  });

  it("validateName should check casing", () => {
    expect(validateName("foo-bar", "kebab-case")).toBe(true);
    expect(validateName("fooBar", "kebab-case")).toBe(false);
    expect(validateName("FooBar", "PascalCase")).toBe(true);
    expect(validateName("foo_bar", "snake_case")).toBe(true);
    expect(validateName("FOO_BAR", "SCREAMING_SNAKE_CASE")).toBe(true);
    expect(validateName("foo", "kebab-case | snake_case")).toBe(true);

    // Test stem validation
    expect(validateName("AGENTS", "SCREAMING_SNAKE_CASE")).toBe(true);
  });

  it("validateName should handle regex", () => {
    expect(validateName("use-js-instead", "regex:use-js-instead")).toBe(true);
  });

  it("checkFile should validate directories and filename without extension", () => {
    const config = {
      ls: {
        ".dir": "kebab-case",
        ".md": "kebab-case | SCREAMING_SNAKE_CASE",
        ".ts": "kebab-case",
      },
      ignore: [],
    };

    // Valid path (stem "AGENTS" matches SCREAMING_SNAKE_CASE)
    expect(checkFile("AGENTS.md", config)).toEqual([]);

    // Valid path (stem "my-file" matches kebab-case)
    expect(checkFile("src/my-file.ts", config)).toEqual([]);

    // Invalid directory
    expect(checkFile("MyPkg/src/index.ts", config)).toEqual([
      'Directory "MyPkg" does not match rules: kebab-case',
    ]);

    // Invalid filename (stem "Index" does not match kebab-case)
    expect(checkFile("src/Index.ts", config)).toEqual([
      'File "Index.ts" (stem: "Index") does not match rules: kebab-case',
    ]);
  });
});
