import { describe, expect, it } from "vitest";
import type { KnownKeysOnly } from "../../../src/utils/types";

describe("KnownKeysOnly Type Helper", () => {
  it("should enforce exhaustive destructuring on known keys", () => {
    // 1. Define a type with known keys AND an index signature (like z.looseObject)
    type LooseType = {
      id: string;
      value: number;
      [key: string]: unknown;
    };

    // 2. Define a subset of those keys (Simulating destructuring rest)
    // In practice, `rest` would be `LooseType`, but with some keys omitted.
    // Here we test KnownKeysOnly<LooseType> directly.

    // A valid object with ONLY known keys should be assignable
    const valid: KnownKeysOnly<LooseType> = {
      id: "abc",
      value: 123,
    };
    expect(valid).toBeDefined();

    // An object with unknown keys should NOT be assignable (compile-time check)
    // We use @ts-expect-error to verify this behavior
    const invalid: KnownKeysOnly<LooseType> = {
      id: "abc",
      value: 123,
      // @ts-expect-error: Unknown keys should be rejected
      unknownKey: "should fail",
    };
    expect(invalid).toBeDefined();
  });

  it("should work with partial types", () => {
    type LooseType = {
      id: string;
      optional?: number;
      [key: string]: unknown;
    };

    const valid: KnownKeysOnly<LooseType> = {
      id: "abc",
    };
    expect(valid).toBeDefined();

    const invalidWithOptional: KnownKeysOnly<LooseType> = {
      id: "abc",
      // @ts-expect-error: Unknown keys should be rejected
      unknownKey: "should fail",
    };
    expect(invalidWithOptional).toBeDefined();
  });

  it("should work with assigned indices", () => {
    type LooseType = {
      id: string;
      value: number;
      [key: string]: unknown;
    };

    const valid: LooseType = {
      id: "abc",
      value: 123,
      somethingElse: "should be ok",
    };
    expect(valid).toBeDefined();

    const knownValid: KnownKeysOnly<LooseType> = {
      id: "abc",
      value: 123,
    };
    expect(knownValid).toBeDefined();
  });
});
