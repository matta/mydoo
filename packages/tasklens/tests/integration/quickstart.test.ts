import { describe, it, expect } from "vitest";
import { TunnelStore } from "../../src/store";

describe("Quickstart Integration", () => {
  it("should run the quickstart scenario", () => {
    // 1. Initialize Store
    const store = new TunnelStore();

    // 2. Create Data
    const rootGoal = store.createTask({
      title: "Work",
      desiredCredits: 100,
    });

    const task = store.createTask({
      title: "Email",
      parentId: rootGoal.id,
      creditIncrement: 1.0,
    });

    // 3. Update Priorities
    store.recalculateScores({ placeId: "All" });

    // 4. Get Todo List
    const todos = store.getTodoList({ currentTime: Date.now() });

    // Email should be visible and sorted
    expect(todos.length).toBeGreaterThan(0);
    expect(todos[0].id).toBe(task.id);
    expect(todos[0].priority).toBeGreaterThan(0);

    // Verify Work (Root) is hidden (Container Visibility Pass 7)
    // Work has a visible child (Email), so Work should be hidden.
    const workTask = store.getTask(rootGoal.id);
    expect(workTask?.visibility).toBe(false);
    expect(workTask?.priority).toBe(0.0);
  });
});
