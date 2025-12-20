import * as Automerge from "@automerge/automerge";
import {
  Task,
  TunnelState,
  TaskID,
  TaskStatus,
  Schedule,
  ViewFilter,
  Context,
  ANYWHERE_PLACE_ID,
} from "./types";
import { recalculateScores as runRecalculateScores } from "./algorithm";
import { getCurrentTimestamp, daysToMilliseconds } from "./utils/time";

export class TunnelStore {
  public doc: Automerge.Doc<TunnelState>;

  constructor(initialState?: TunnelState) {
    if (initialState) {
      if (ANYWHERE_PLACE_ID in initialState.places) {
        throw new Error(
          `'${ANYWHERE_PLACE_ID}' is a reserved Place ID and cannot be defined.`,
        );
      }
      this.doc = Automerge.from(initialState);
    } else {
      this.doc = Automerge.from({
        tasks: {},
        places: {},
        nextTaskId: 1,
        nextPlaceId: 1,
      });
    }
  }

  get state(): TunnelState {
    return this.doc;
  }

  // Helper to get task from a given doc state
  private _getTaskFromDoc(docState: TunnelState, id: TaskID): Task | undefined {
    return docState.tasks[id];
  }

  // Helper to get children from a given doc state
  private _getChildrenFromDoc(
    docState: TunnelState,
    parentId: TaskID | null,
  ): Task[] {
    return Object.values(docState.tasks).filter(
      (task) => task.parentId === parentId,
    );
  }

  // Helper to get ancestors from a given doc state
  private _getAncestorsFromDoc(docState: TunnelState, id: TaskID): Task[] {
    const ancestors: Task[] = [];
    let currentTask = this._getTaskFromDoc(docState, id);
    while (currentTask && currentTask.parentId !== null) {
      const parent = this._getTaskFromDoc(docState, currentTask.parentId);
      if (parent) {
        ancestors.unshift(parent); // Add to the beginning to maintain root-to-child order
        currentTask = parent;
      } else {
        // Parent not found, break to prevent infinite loop
        break;
      }
    }
    return ancestors;
  }

  // --- CRUD Operations ---
  getTask(id: TaskID): Task | undefined {
    return this._getTaskFromDoc(this.doc, id);
  }

  // Public Getters for hierarchy
  getChildren(parentId: TaskID | null): Task[] {
    return this._getChildrenFromDoc(this.doc, parentId);
  }

  getAncestors(id: TaskID): Task[] {
    return this._getAncestorsFromDoc(this.doc, id);
  }

  private _validateTaskProps(props: Partial<Task>): void {
    if (props.credits !== undefined && props.credits < 0) {
      throw new Error("Credits cannot be negative.");
    }
    if (props.desiredCredits !== undefined && props.desiredCredits < 0) {
      throw new Error("DesiredCredits cannot be negative.");
    }
    if (props.creditIncrement !== undefined && props.creditIncrement < 0) {
      throw new Error("CreditIncrement cannot be negative.");
    }
    if (props.schedule?.leadTime !== undefined && props.schedule.leadTime < 0) {
      throw new Error("LeadTime cannot be negative.");
    }
    if (
      props.importance !== undefined &&
      (props.importance < 0 || props.importance > 1)
    ) {
      throw new Error("Importance must be between 0.0 and 1.0.");
    }
  }

  private _getTaskDepthFromDoc(docState: TunnelState, taskId: TaskID): number {
    let depth = 0;
    let currentTask = this._getTaskFromDoc(docState, taskId);
    while (currentTask && currentTask.parentId !== null) {
      depth++;
      currentTask = this._getTaskFromDoc(docState, currentTask.parentId);
      if (depth > 20) {
        // Max hierarchy depth check during traversal to prevent infinite loops and validate
        throw new Error(
          "Maximum hierarchy depth (20) exceeded during traversal.",
        );
      }
    }
    return depth;
  }

  createTask(props: Partial<Task>): Task {
    let newTask: Task | undefined;
    this.doc = Automerge.change(this.doc, "Create task", (doc) => {
      this._validateTaskProps(props);

      const newTaskIdNum = doc.nextTaskId;
      const newTaskId = String(newTaskIdNum); // Generate string ID

      if (props.parentId !== null && props.parentId !== undefined) {
        const parentTask = this._getTaskFromDoc(doc, props.parentId);
        if (!parentTask) {
          throw new Error(`Parent task with ID ${props.parentId} not found.`);
        }
        const parentDepth = this._getTaskDepthFromDoc(doc, props.parentId);
        if (parentDepth >= 20) {
          throw new Error(
            "Cannot create task: parent already at maximum hierarchy depth (20).",
          );
        }
      }

      const defaultSchedule: Schedule = {
        type: "Once",
        dueDate: null,
        leadTime: daysToMilliseconds(7),
      }; // 7 days in ms
      newTask = {
        id: newTaskId, // Assign string ID
        title: props.title ?? "New Task",
        parentId: props.parentId ?? null,
        placeId: props.placeId ?? null, // Default to null for now, will inherit later
        status: props.status ?? TaskStatus.Pending,
        importance: props.importance ?? 1.0,
        creditIncrement: props.creditIncrement ?? 1.0,
        credits: props.credits ?? 0.0,
        desiredCredits: props.desiredCredits ?? 0.0,
        creditsTimestamp: props.creditsTimestamp ?? getCurrentTimestamp(),
        priorityTimestamp: props.priorityTimestamp ?? getCurrentTimestamp(),
        schedule: props.schedule ?? defaultSchedule,
        isSequential: props.isSequential ?? false,
      };

      doc.tasks[newTaskId] = newTask; // Store with string key
      doc.nextTaskId = newTaskIdNum + 1; // Increment number counter
    });

    if (!newTask) {
      throw new Error("Failed to create task.");
    }
    const createdTask = this._getTaskFromDoc(this.doc, newTask.id);
    if (!createdTask) {
      throw new Error("Failed to retrieve created task.");
    }
    return createdTask;
  }

  updateTask(id: TaskID, props: Partial<Task>): Task {
    let updatedTask: Task | undefined;
    this.doc = Automerge.change(this.doc, `Update task ${id}`, (doc) => {
      const existingTask = this._getTaskFromDoc(doc, id);
      if (!existingTask) {
        throw new Error(`Task with ID ${id} not found.`);
      }

      // Perform validation on props before applying
      this._validateTaskProps(props);

      // Check for parentId change and associated depth limits
      if (
        props.parentId !== undefined &&
        props.parentId !== existingTask.parentId
      ) {
        if (props.parentId !== null) {
          const parentTask = this._getTaskFromDoc(doc, props.parentId);
          if (!parentTask) {
            throw new Error(`Parent task with ID ${props.parentId} not found.`);
          }
          const parentDepth = this._getTaskDepthFromDoc(doc, props.parentId);
          if (parentDepth >= 20) {
            throw new Error(
              "Cannot move task: new parent already at maximum hierarchy depth (20).",
            );
          }
        }
        // Also check if moving this task would make its *own* subtree too deep
        // This is a more complex check, typically done by temporarily moving and checking max depth of subtree.
        // For now, only checking parent depth. Full check can be added later if needed.
      }

      // Apply updates (excluding ID which is immutable)
      // Use Object.entries and Reflect.set for clean dynamic assignment
      for (const [key, value] of Object.entries(props)) {
        if (key !== "id") {
          Reflect.set(existingTask, key, value);
        }
      }
      updatedTask = existingTask; // Now existingTask is the updated one
    });
    if (!updatedTask) {
      throw new Error("Failed to update task.");
    }
    return updatedTask;
  }

  // Credit Decay Algorithm (from Section 4.2 of ALGORITHM.md)
  private _applyCreditDecay(
    credits: number,
    creditsTimestamp: number,
    currentTime: number,
  ): number {
    const halfLifeMillis = daysToMilliseconds(7); // 7 days in milliseconds

    const timeDelta = currentTime - creditsTimestamp;
    return credits * Math.pow(0.5, timeDelta / halfLifeMillis);
  }

  completeTask(id: TaskID): void {
    this.doc = Automerge.change(this.doc, `Complete task ${id}`, (doc) => {
      const taskToComplete = this._getTaskFromDoc(doc, id);
      if (!taskToComplete) {
        throw new Error(`Task with ID ${id} not found.`);
      }
      if (taskToComplete.status === TaskStatus.Done) {
        return; // Already done, no credit attribution needed
      }

      // Mark task as done
      taskToComplete.status = TaskStatus.Done;

      // 1. Identify Path: Ancestral path from task to root
      const ancestralPath = this._getAncestorsFromDoc(doc, id);
      const pathNodes = [...ancestralPath, taskToComplete]; // Include the task itself

      const currentTime = getCurrentTimestamp();

      // 2. Apply Decay (Pre-update) and 3. Add Effort, 4. Update Timestamp
      pathNodes.forEach((node) => {
        const decayedCredits = this._applyCreditDecay(
          node.credits,
          node.creditsTimestamp,
          currentTime,
        );
        node.credits = decayedCredits + node.creditIncrement;
        node.creditsTimestamp = currentTime;
      });

      // 5. Recurring Tasks: The spec says "If Task T repeats, the effort is added to the cumulative total; the existing history is not reset."
      // This implies that the status should not be 'Done' for recurring tasks but rather reset/advanced.
      // For now, if it's a recurring task, we'll just mark it pending again after credit attribution.
      // The specific logic for advancing a recurring schedule is not fully detailed here.
      if (taskToComplete.schedule.type === "Recurring") {
        taskToComplete.status = TaskStatus.Pending; // Reset for next cycle
        // TODO: Advance recurring schedule (dueDate, etc.)
      }
    });
  }

  // --- Algorithm Operations ---
  recalculateScores(viewFilter: ViewFilter, context?: Context): void {
    // Pass the store instance and parameters to the orchestrator
    runRecalculateScores(this, viewFilter, context);
  }

  getTodoList(_context: Context): Task[] {
    const allTasks = Object.values(this.doc.tasks);
    const visibleTasks = allTasks.filter(
      (t) => t.visibility && (t.priority ?? 0) > 0.001,
    );

    // Calculate DFS Pre-Order Index for tie-breaking
    const dfsOrder = new Map<string, number>();
    let index = 0;

    // Helper to traverse
    const visit = (taskId: string) => {
      dfsOrder.set(taskId, index++);
      // Get children, sort by ID for deterministic traversal
      const children = this._getChildrenFromDoc(this.doc, taskId).sort(
        (a, b) => {
          return a.id.localeCompare(b.id);
        },
      );
      children.forEach((c) => {
        visit(c.id);
      });
    };

    // Find roots, sort by ID, traverse
    const roots = allTasks
      .filter((t) => t.parentId === null)
      .sort((a, b) => {
        return a.id.localeCompare(b.id);
      });
    roots.forEach((r) => {
      visit(r.id);
    });

    return visibleTasks.sort((a, b) => {
      const priorityDiff = (b.priority ?? 0) - (a.priority ?? 0);
      if (Math.abs(priorityDiff) > 0.000001) {
        return priorityDiff;
      }
      const orderA = dfsOrder.get(a.id) ?? Infinity;
      const orderB = dfsOrder.get(b.id) ?? Infinity;
      return orderA - orderB;
    });
  }

  // --- Persistence ---
  save(): Uint8Array {
    return Automerge.save(this.doc);
  }

  static load(data: Uint8Array): TunnelStore {
    const doc = Automerge.load<TunnelState>(data);
    const store = new TunnelStore();
    store.doc = doc;
    return store;
  }
}
