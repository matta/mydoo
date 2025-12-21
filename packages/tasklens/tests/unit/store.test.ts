import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { TunnelStore } from "../../src/store";
import { type Task, TaskStatus } from "../../src/types";
import {
  mockCurrentTimestamp,
  resetCurrentTimestampMock,
} from "../../src/utils/time";

describe("TunnelStore", () => {
  let store: TunnelStore;

  beforeEach(() => {
    store = new TunnelStore();
    resetCurrentTimestampMock(); // Ensure time is reset before each test
  });

  afterEach(() => {
    resetCurrentTimestampMock();
  });

  it("should initialize with an empty state and default next IDs", () => {
    const state = store.state;
    expect(state.tasks).toEqual({});
    expect(state.places).toEqual({});
    expect(state.nextTaskId).toBe(1);
    expect(state.nextPlaceId).toBe(1);
  });

  it("should allow initializing with a provided state", () => {
    const initialStoreState = {
      tasks: {
        "1": {
          id: "1",
          title: "Existing Task",
          parentId: null,
          placeId: null,
          status: TaskStatus.Pending,
          importance: 1,
          creditIncrement: 1,
          credits: 0,
          desiredCredits: 0,
          creditsTimestamp: 0,
          priorityTimestamp: 0,
          schedule: { type: "Once" as const, dueDate: null, leadTime: 0 },
          isSequential: false,
        },
      },
      places: {},
      nextTaskId: 2,
      nextPlaceId: 1,
    };
    store = new TunnelStore(initialStoreState);
    expect(store.state.nextTaskId).toBe(2);
    expect(store.getTask("1")).toEqual(initialStoreState.tasks["1"]);
  });

  describe("createTask", () => {
    it("should create a new task with default values", () => {
      const currentTime = 1678886400000; // March 15, 2023 00:00:00 UTC
      mockCurrentTimestamp(currentTime);

      const task = store.createTask({ title: "My New Task" });

      expect(task).toBeDefined();
      expect(task.id).toBe("1");
      expect(task.title).toBe("My New Task");
      expect(task.status).toBe(TaskStatus.Pending);
      expect(task.creditsTimestamp).toBe(currentTime);
      expect(store.state.nextTaskId).toBe(2);
      expect(store.getTask("1")).toEqual(task);
    });

    it("should create a child task and update nextTaskId", () => {
      const parent = store.createTask({ title: "Parent Task" });
      const child = store.createTask({
        title: "Child Task",
        parentId: parent.id,
      });

      expect(child.id).toBe("2");
      expect(child.parentId).toBe(parent.id);
      expect(store.state.nextTaskId).toBe(3);
      expect(store.getTask("2")).toEqual(child);
    });

    it("should throw an error for negative creditIncrement", () => {
      expect(() =>
        store.createTask({ title: "Invalid Task", creditIncrement: -0.5 }),
      ).toThrow("CreditIncrement cannot be negative.");
    });

    it("should throw an error for importance outside 0-1 range", () => {
      expect(() =>
        store.createTask({ title: "Invalid Task", importance: 1.5 }),
      ).toThrow("Importance must be between 0.0 and 1.0.");
      expect(() =>
        store.createTask({ title: "Invalid Task", importance: -0.1 }),
      ).toThrow("Importance must be between 0.0 and 1.0.");
    });

    it("should throw an error if parentId does not exist", () => {
      expect(() =>
        store.createTask({ title: "Orphan Task", parentId: "999" }),
      ).toThrow("Parent task with ID 999 not found.");
    });

    it("should throw an error if hierarchy depth limit is exceeded", () => {
      let parentId: string | null = null;
      for (let i = 0; i < 20; i++) {
        // Loop 20 times to create tasks from depth 0 to 19
        const task = store.createTask({
          title: `Task ${i.toString()}`,
          parentId,
        });
        parentId = task.id;
      }
      // Now parentId is a task at depth 19. Creating a child will make it depth 20.
      // This should SUCCEED.
      const taskAtDepth20 = store.createTask({ title: "Depth 20", parentId });
      expect(taskAtDepth20).toBeDefined();

      // Now try to create a child of the task at depth 20 (i.e., depth 21)
      expect(() =>
        store.createTask({ title: "Too Deep", parentId: taskAtDepth20.id }),
      ).toThrow(
        "Cannot create task: parent already at maximum hierarchy depth (20).",
      );
    });
  });

  describe("updateTask", () => {
    let initialTask: Task;

    beforeEach(() => {
      initialTask = store.createTask({
        title: "Original Task",
        importance: 0.5,
      });
    });

    it("should update an existing task field", () => {
      const updatedTitle = "Updated Title";
      const updatedTask = store.updateTask(initialTask.id, {
        title: updatedTitle,
      });

      expect(updatedTask.id).toBe(initialTask.id);
      expect(updatedTask.title).toBe(updatedTitle);
      expect(store.getTask(initialTask.id)?.title).toBe(updatedTitle);
    });

    it("should throw an error if task to update does not exist", () => {
      expect(() => store.updateTask("999", { title: "Non Existent" })).toThrow(
        "Task with ID 999 not found.",
      );
    });

    it("should prevent updating the task ID", () => {
      const attemptUpdate = { id: "999", title: "New ID" };
      const updatedTask = store.updateTask(
        initialTask.id,
        attemptUpdate as Partial<Task>,
      );
      expect(updatedTask.id).toBe(initialTask.id); // ID should not change
      expect(updatedTask.title).toBe("New ID"); // Other props update
    });

    it("should throw an error for negative desiredCredits during update", () => {
      expect(() =>
        store.updateTask(initialTask.id, { desiredCredits: -10 }),
      ).toThrow("DesiredCredits cannot be negative.");
    });

    it("should throw an error when moving to a non-existent parent", () => {
      expect(() =>
        store.updateTask(initialTask.id, { parentId: "999" }),
      ).toThrow("Parent task with ID 999 not found.");
    });

    it("should throw an error when moving to a parent that exceeds depth limit", () => {
      let parentId: string | null = null;
      for (let i = 0; i < 20; i++) {
        const task = store.createTask({
          title: `Task ${i.toString()}`,
          parentId,
        });
        parentId = task.id;
      }
      const taskAtDepth20 = store.createTask({
        title: "Depth 20 Parent",
        parentId,
      });

      // Create a task that we'll try to move under the depth 20 parent
      const taskToMove = store.createTask({
        title: "Task to Move",
        parentId: null,
      }); // Initially root

      // Try to move taskToMove (depth 0) under the taskAtDepth20 (depth 20)
      expect(() =>
        store.updateTask(taskToMove.id, { parentId: taskAtDepth20.id }),
      ).toThrow(
        "Cannot move task: new parent already at maximum hierarchy depth (20).",
      );
    });

    it("should allow setting parentId to null (making it a root task)", () => {
      const childTask = store.createTask({
        title: "Child",
        parentId: initialTask.id,
      });
      const updatedChild = store.updateTask(childTask.id, { parentId: null });
      expect(updatedChild.parentId).toBeNull();
      expect(store.getTask(childTask.id)?.parentId).toBeNull();
    });
  });
});
