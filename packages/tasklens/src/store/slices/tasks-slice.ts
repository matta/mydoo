import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { getPrioritizedTasks } from "../../domain/priority";
import type { TunnelState } from "../../types/persistence";
import type { ComputedTask, TaskID } from "../../types/ui";

/**
 * Redux state for the tasks slice.
 *
 * @property entities - Normalized map of TaskID to ComputedTask. These references
 *   are stabilized to prevent unnecessary re-renders.
 * @property rootTaskIds - Ordered list of top-level task IDs (tree view order).
 * @property todoListIds - Ordered list of task IDs representing the prioritized "Do" list.
 * @property lastProxyDoc - The most recent raw Automerge Proxy, used for reference comparison.
 */
export interface TasksState {
  entities: Record<TaskID, ComputedTask>;
  rootTaskIds: TaskID[];
  todoListIds: TaskID[];
  lastProxyDoc: TunnelState | null;
}

const initialState: TasksState = {
  entities: {},
  rootTaskIds: [],
  todoListIds: [],
  lastProxyDoc: null,
};

/**
 * Payload for the syncDoc thunk.
 *
 * @property proxyDoc - The raw Automerge Proxy object, used for reference stability checks.
 * @property parsedDoc - The Zod-validated POJO, used for data access.
 */
interface SyncDocPayload {
  proxyDoc: TunnelState;
  parsedDoc: TunnelState;
}

/**
 * Syncs the Redux store with a new Automerge document state.
 *
 * It runs the prioritization algorithm and then performs a reference stabilization
 * pass. The raw Automerge Proxy is used only for reference comparison; all data
 * access uses the Zod-parsed POJO.
 *
 * NOTE: Validation happens at the boundary (TaskLensProvider), not here.
 */
export const syncDoc = createAsyncThunk(
  "tasks/syncDoc",
  async (
    { proxyDoc: proxy, parsedDoc: parsed }: SyncDocPayload,
    { getState },
  ) => {
    const state = (getState() as { tasks: TasksState }).tasks;
    const oldProxyDoc = state.lastProxyDoc;
    const oldEntities = state.entities;

    // 1. Get ALL tasks with computed properties for entities
    // We call getPrioritizedTasks with inclusive options to get the full map.
    const allTasksComputed = getPrioritizedTasks(
      parsed,
      {},
      {
        includeHidden: true,
        mode: "plan-outline",
      },
    );

    const newEntities: Record<TaskID, ComputedTask> = {};
    for (const task of allTasksComputed) {
      const id = task.id;
      // Reference Stability Check:
      // Compare the raw Automerge Proxy references to detect changes.
      // If the task node reference hasn't changed in the raw document,
      // we can reuse the existing ComputedTask entity to save React renders.
      const persistenceId = id;
      if (
        oldProxyDoc &&
        oldProxyDoc.tasks[persistenceId] === proxy.tasks[persistenceId] &&
        oldEntities[id]
      ) {
        newEntities[id] = oldEntities[id];
      } else {
        newEntities[id] = task;
      }
    }

    // 2. Get the prioritized list for the "Do" view
    const prioritizedTasks = getPrioritizedTasks(parsed);
    const todoListIds = prioritizedTasks.map((t) => t.id);

    return {
      entities: newEntities,
      rootTaskIds: parsed.rootTaskIds,
      todoListIds,
      newProxyDoc: proxy,
    };
  },
);

const tasksSlice = createSlice({
  name: "tasks",
  initialState,
  reducers: {},
  extraReducers: (builder) => {
    builder.addCase(syncDoc.fulfilled, (state, action) => {
      state.entities = action.payload.entities;
      state.rootTaskIds = action.payload.rootTaskIds;
      state.todoListIds = action.payload.todoListIds;
      state.lastProxyDoc = action.payload.newProxyDoc;
    });
  },
});

export default tasksSlice.reducer;
