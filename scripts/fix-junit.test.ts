import { describe, expect, it } from "vitest";
import { transformJunit } from "./fix-junit.js";

describe("transformJunit", () => {
  const packageDir = "crates/tasklens-ui";

  it("adds file attribute to testcase with classname", () => {
    const input =
      '<testcase name="Alice and Bob" classname="tests/e2e/sync.spec.ts" time="30.264"></testcase>';
    const expected =
      '<testcase name="Alice and Bob" classname="tests/e2e/sync.spec.ts" time="30.264" file="crates/tasklens-ui/tests/e2e/sync.spec.ts" ></testcase>';
    expect(transformJunit(input, packageDir)).toBe(expected);
  });

  it("handles self-closing testcase tags", () => {
    const input =
      '<testcase name="Smoke Test" classname="tests/e2e/smoke.spec.ts" time="2.078" />';
    const expected =
      '<testcase name="Smoke Test" classname="tests/e2e/smoke.spec.ts" time="2.078" file="crates/tasklens-ui/tests/e2e/smoke.spec.ts" />';
    expect(transformJunit(input, packageDir)).toBe(expected);
  });

  it("skips if file attribute is already present", () => {
    const input =
      '<testcase name="Skip Me" classname="foo.ts" file="already/here.ts" />';
    expect(transformJunit(input, packageDir)).toBe(input);
  });

  it("skips if classname is missing", () => {
    const input = '<testcase name="No Classname" time="1.0" />';
    expect(transformJunit(input, packageDir)).toBe(input);
  });

  it("handles multiple testcases in one string", () => {
    const input = `
<testsuite>
  <testcase name="Test 1" classname="file1.ts" />
  <testcase name="Test 2" classname="file2.ts" />
</testsuite>`.trim();
    const output = transformJunit(input, packageDir);
    expect(output).toContain('file="crates/tasklens-ui/file1.ts"');
    expect(output).toContain('file="crates/tasklens-ui/file2.ts"');
  });

  it("preserves other attributes and content", () => {
    const input =
      '<testcase name="Complex" classname="test.ts"><system-out>Some log</system-out></testcase>';
    const output = transformJunit(input, packageDir);
    expect(output).toContain('file="crates/tasklens-ui/test.ts"');
    expect(output).toContain("<system-out>Some log</system-out>");
  });
});
