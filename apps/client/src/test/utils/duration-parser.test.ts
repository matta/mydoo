import { describe, expect, it } from "vitest";
import { durationToMs, parseDuration } from "./duration-parser";

describe("parseDuration", () => {
  describe("valid inputs", () => {
    it("should parse singular day", () => {
      const result = parseDuration("1 day");
      expect(result).toEqual({ value: 1, rawUnit: "day", uiUnit: "Days" });
    });

    it("should parse plural days", () => {
      const result = parseDuration("3 days");
      expect(result).toEqual({ value: 3, rawUnit: "days", uiUnit: "Days" });
    });

    it("should parse singular hour", () => {
      const result = parseDuration("1 hour");
      expect(result).toEqual({ value: 1, rawUnit: "hour", uiUnit: "Hours" });
    });

    it("should parse plural hours", () => {
      const result = parseDuration("2 hours");
      expect(result).toEqual({ value: 2, rawUnit: "hours", uiUnit: "Hours" });
    });

    it('should parse hour abbreviation "hr"', () => {
      const result = parseDuration("1 hr");
      expect(result).toEqual({ value: 1, rawUnit: "hr", uiUnit: "Hours" });
    });

    it('should parse hour abbreviation "hrs"', () => {
      const result = parseDuration("5 hrs");
      expect(result).toEqual({ value: 5, rawUnit: "hrs", uiUnit: "Hours" });
    });

    it("should parse singular minute", () => {
      const result = parseDuration("1 minute");
      expect(result).toEqual({
        value: 1,
        rawUnit: "minute",
        uiUnit: "Minutes",
      });
    });

    it("should parse plural minutes", () => {
      const result = parseDuration("30 minutes");
      expect(result).toEqual({
        value: 30,
        rawUnit: "minutes",
        uiUnit: "Minutes",
      });
    });

    it('should parse minute abbreviation "min"', () => {
      const result = parseDuration("15 min");
      expect(result).toEqual({ value: 15, rawUnit: "min", uiUnit: "Minutes" });
    });

    it('should parse minute abbreviation "mins"', () => {
      const result = parseDuration("45 mins");
      expect(result).toEqual({ value: 45, rawUnit: "mins", uiUnit: "Minutes" });
    });

    it("should handle mixed case units", () => {
      const result = parseDuration("2 HOURS");
      expect(result).toEqual({ value: 2, rawUnit: "hours", uiUnit: "Hours" });
    });

    it("should handle extra whitespace", () => {
      const result = parseDuration("  3   days  ");
      expect(result).toEqual({ value: 3, rawUnit: "days", uiUnit: "Days" });
    });
  });

  describe("invalid inputs", () => {
    it("should throw on empty string", () => {
      expect(() => parseDuration("")).toThrow(
        'Invalid duration format: "". Expected format: "<number> <unit>"',
      );
    });

    it("should throw on missing number", () => {
      expect(() => parseDuration("days")).toThrow(
        'Invalid duration format: "days". Expected format: "<number> <unit>"',
      );
    });

    it("should throw on missing unit", () => {
      expect(() => parseDuration("3")).toThrow(
        'Invalid duration format: "3". Expected format: "<number> <unit>"',
      );
    });

    it("should throw on non-numeric value", () => {
      expect(() => parseDuration("abc days")).toThrow(
        'Invalid duration value: "abc" in "abc days". Expected a number.',
      );
    });

    it("should throw on unrecognized unit", () => {
      expect(() => parseDuration("3 weeks")).toThrow(
        /Unrecognized duration unit: "weeks" in "3 weeks"\. Valid units:/,
      );
    });

    it("should throw on too many parts", () => {
      expect(() => parseDuration("3 days ago")).toThrow(
        'Invalid duration format: "3 days ago". Expected format: "<number> <unit>"',
      );
    });
  });
});

describe("durationToMs", () => {
  describe("minutes conversion", () => {
    it("should convert 1 minute to milliseconds", () => {
      expect(durationToMs("1 minute")).toBe(60 * 1000);
    });

    it("should convert 30 minutes to milliseconds", () => {
      expect(durationToMs("30 minutes")).toBe(30 * 60 * 1000);
    });

    it("should convert abbreviated minutes to milliseconds", () => {
      expect(durationToMs("15 min")).toBe(15 * 60 * 1000);
    });
  });

  describe("hours conversion", () => {
    it("should convert 1 hour to milliseconds", () => {
      expect(durationToMs("1 hour")).toBe(60 * 60 * 1000);
    });

    it("should convert 8 hours to milliseconds", () => {
      expect(durationToMs("8 hours")).toBe(8 * 60 * 60 * 1000);
    });

    it("should convert abbreviated hours to milliseconds", () => {
      expect(durationToMs("2 hrs")).toBe(2 * 60 * 60 * 1000);
    });
  });

  describe("days conversion", () => {
    it("should convert 1 day to milliseconds", () => {
      expect(durationToMs("1 day")).toBe(24 * 60 * 60 * 1000);
    });

    it("should convert 3 days to milliseconds", () => {
      expect(durationToMs("3 days")).toBe(3 * 24 * 60 * 60 * 1000);
    });
  });

  describe("error handling", () => {
    it("should throw on invalid format", () => {
      expect(() => durationToMs("invalid")).toThrow(
        'Invalid duration format: "invalid". Expected format: "<number> <unit>"',
      );
    });

    it("should throw on unrecognized unit", () => {
      expect(() => durationToMs("2 weeks")).toThrow(
        /Unrecognized duration unit: "weeks"/,
      );
    });
  });
});
