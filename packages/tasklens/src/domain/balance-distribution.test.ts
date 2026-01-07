import { describe, expect, it } from "vitest";
import type { TaskID } from "../types";
import {
  type BalanceItemSimple,
  distributeCredits,
} from "./balance-distribution";

// Helper to create simple items
const createItem = (idStr: string, credits: number): BalanceItemSimple => ({
  id: idStr as TaskID,
  desiredCredits: credits,
});

describe("distributeCredits", () => {
  it("should return empty list if items are empty", () => {
    const updates = distributeCredits("1" as TaskID, 0.5, []);
    expect(updates).toHaveLength(0);
  });

  it("should force single item to 1.0", () => {
    const items = [createItem("1", 0.5)];
    // Try to set it to 0.8
    const updates = distributeCredits("1" as TaskID, 0.8, items);
    expect(updates).toHaveLength(1);
    expect(updates[0]?.desiredCredits).toBe(1.0);
  });

  it("should do nothing if change is negligible", () => {
    const items = [createItem("1", 0.5), createItem("2", 0.5)];
    const updates = distributeCredits("1" as TaskID, 0.5, items);
    expect(updates).toHaveLength(0);
  });

  describe("Increasing Credits (Taking from others)", () => {
    it("should decrease other items proportionally to their surplus", () => {
      // min for n=3 is 0.03
      // Max for n=3 is 3 - 2*0.03 = 2.94

      // Setup: 3 items, equal credits (1.0 each)
      const items = [
        createItem("1", 1.0),
        createItem("2", 1.0),
        createItem("3", 1.0),
      ];

      // Increase '1' by 0.2 -> 1.2
      // Delta = 0.2
      // Others ('2', '3') have surplus (1.0 - 0.03 = 0.97 each)
      // Total surplus = 1.94
      // '2' tax = 0.2 * (0.97 / 1.94) = 0.1
      // '3' tax = 0.2 * (0.97 / 1.94) = 0.1

      const updates = distributeCredits("1" as TaskID, 1.2, items);

      expect(updates).toHaveLength(3);
      const u1 = updates.find((u) => u.id === ("1" as TaskID));
      const u2 = updates.find((u) => u.id === ("2" as TaskID));
      const u3 = updates.find((u) => u.id === ("3" as TaskID));

      expect(u1?.desiredCredits).toBeCloseTo(1.2);
      expect(u2?.desiredCredits).toBeCloseTo(0.9);
      expect(u3?.desiredCredits).toBeCloseTo(0.9);
    });

    it("should not decrease items below minimum", () => {
      // Setup: '1' at 1.0. '2' at min (0.03). '3' at 1.97.
      // min for n=3 is 0.03.
      const items = [
        createItem("1", 1.0),
        createItem("2", 0.03),
        createItem("3", 1.97),
      ];

      // Increase '1' by 0.5 -> 1.5
      // Delta = 0.5
      // '2' surplus = 0
      // '3' surplus = 1.94
      // Total surplus = 1.94

      // '2' should pay 0 tax.
      // '3' should pay all tax (0.5).

      const updates = distributeCredits("1" as TaskID, 1.5, items);

      const u2 = updates.find((u) => u.id === ("2" as TaskID));
      const u3 = updates.find((u) => u.id === ("3" as TaskID));

      // u2 might not even be in updates if it didn't change, OR it might be there with same value?
      // The logic iterates all `otherItems` and pushes updates.
      // tax for '2' = 0.5 * (0 / 1.94) = 0.
      // new value = 0.03 - 0 = 0.03.

      expect(u2?.desiredCredits).toBeCloseTo(0.03);
      expect(u3?.desiredCredits).toBeCloseTo(1.47); // 1.97 - 0.5
    });
  });

  describe("Decreasing Credits (Giving to others)", () => {
    it("should distribute credits to others proportionally to their weight", () => {
      // Setup: 3 items
      // '1' = 1.0
      // '2' = 1.0
      // '3' = 1.0

      const items = [
        createItem("1", 1.0),
        createItem("2", 1.0),
        createItem("3", 1.0),
      ];

      // Decrease '1' by 0.2 -> 0.8
      // Budget to add = 0.2
      // Total weight of others = 2.0
      // '2' share = 0.2 * (1.0 / 2.0) = 0.1
      // '3' share = 0.2 * (1.0 / 2.0) = 0.1

      const updates = distributeCredits("1" as TaskID, 0.8, items);

      const u1 = updates.find((u) => u.id === ("1" as TaskID));
      const u2 = updates.find((u) => u.id === ("2" as TaskID));
      const u3 = updates.find((u) => u.id === ("3" as TaskID));

      expect(u1?.desiredCredits).toBeCloseTo(0.8);
      expect(u2?.desiredCredits).toBeCloseTo(1.1);
      expect(u3?.desiredCredits).toBeCloseTo(1.1);
    });

    it("should split equally if others have 0 weight (edge case)", () => {
      // This state is technically invalid per min-constraints but good to test robustness
      const items = [
        createItem("1", 1.0),
        createItem("2", 0),
        createItem("3", 0),
      ];

      // Decrease '1' to 0.8. Add 0.2 to others.
      const updates = distributeCredits("1" as TaskID, 0.8, items);

      const u2 = updates.find((u) => u.id === ("2" as TaskID));
      const u3 = updates.find((u) => u.id === ("3" as TaskID));

      expect(u2?.desiredCredits).toBeCloseTo(0.1);
      expect(u3?.desiredCredits).toBeCloseTo(0.1);
    });
  });
});
