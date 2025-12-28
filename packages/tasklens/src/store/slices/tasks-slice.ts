import {createAsyncThunk, createSlice} from '@reduxjs/toolkit';
import {getPrioritizedTasks} from '../../domain/priority';
import {TunnelStateSchema} from '../../persistence/schemas';
import type {ComputedTask, TaskID, TunnelState} from '../../types';

/**
 * Redux state for the tasks slice.
 *
 * @property entities - Normalized map of TaskID to ComputedTask. These references
 *   are stabilized to prevent unnecessary re-renders.
 * @property todoListIds - Ordered list of task IDs representing the prioritized "Do" list.
 * @property lastDoc - The most recent TunnelState used for reference comparison.
 */
export interface TasksState {
  entities: Record<TaskID, ComputedTask>;
  todoListIds: TaskID[];
  lastDoc: TunnelState | null;
}

const initialState: TasksState = {
  entities: {},
  todoListIds: [],
  lastDoc: null,
};

/**
 * Syncs the Redux store with a new Automerge document state.
 *
 * It runs the prioritization algorithm and then performs a reference stabilization
 * pass.
 *
 * NOTE: This thunk strictly parses the incoming Automerge Document into a POJO
 * via Zod before invoking the domain logic, ensuring no Proxies leak into the algorithm.
 */
export const syncDoc = createAsyncThunk(
  'tasks/syncDoc',
  async (newDocRaw: TunnelState, {getState}) => {
    // Validate and sanitize the document (convert Proxy to POJO).
    // This is crucial because Redux passes the raw Automerge Proxy by default,
    // while the domain logic expects standard JS objects.
    const validation = TunnelStateSchema.safeParse(newDocRaw);
    if (!validation.success) {
      console.warn('[tasks-slice] Invalid document structure ignored.');
      return {
        entities: (getState() as {tasks: TasksState}).tasks.entities,
        todoListIds: (getState() as {tasks: TasksState}).tasks.todoListIds,
        newDoc: (getState() as {tasks: TasksState}).tasks.lastDoc,
      };
    }
    const newDoc = validation.data;

    const state = (getState() as {tasks: TasksState}).tasks;
    const oldDoc = state.lastDoc;
    const oldEntities = state.entities;

    // 1. Run the prioritization algorithm
    const prioritizedTasks = getPrioritizedTasks(newDoc);

    // 2. Stabilize references
    const newEntities: Record<TaskID, ComputedTask> = {};
    const todoListIds: TaskID[] = [];

    for (const task of prioritizedTasks) {
      const id = task.id;
      todoListIds.push(id);

      // If the task exists in the old doc and its reference is identical,
      // and it exists in our current Redux entities, reuse the old entity.
      // NOTE: We check using newDocRaw (the input Automerge Proxy) because it guarantees
      // reference stability for unchanged objects. newDoc (the POJO) is always fresh.
      if (
        oldDoc &&
        oldDoc.tasks[id] === newDocRaw.tasks[id] &&
        oldEntities[id]
      ) {
        newEntities[id] = oldEntities[id];
      } else {
        newEntities[id] = task;
      }
    }

    return {
      entities: newEntities,
      todoListIds,
      newDoc: newDocRaw, // Store the Raw Proxy for next comparison
    };
  },
);

export const tasksSlice = createSlice({
  name: 'tasks',
  initialState,
  reducers: {},
  extraReducers: builder => {
    builder.addCase(syncDoc.fulfilled, (state, action) => {
      state.entities = action.payload.entities;
      state.todoListIds = action.payload.todoListIds;
      state.lastDoc = action.payload.newDoc;
    });
  },
});

export const selectIsReady = (state: {tasks: TasksState}) =>
  state.tasks.lastDoc !== null;

export default tasksSlice.reducer;
