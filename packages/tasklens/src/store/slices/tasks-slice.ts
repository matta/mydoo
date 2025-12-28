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

    // 1. Get ALL tasks with computed properties for entities
    // We call getPrioritizedTasks with inclusive options to get the full map.
    const allTasksComputed = getPrioritizedTasks(
      newDoc,
      {},
      {
        includeHidden: true,
        includeDone: true,
      },
    );

    const newEntities: Record<TaskID, ComputedTask> = {};
    for (const task of allTasksComputed) {
      const id = task.id;
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

    // 2. Get the prioritized list for the "Do" view
    const prioritizedTasks = getPrioritizedTasks(newDoc);
    const todoListIds = prioritizedTasks.map(t => t.id);

    return {
      entities: newEntities,
      todoListIds,
      newDoc: newDocRaw,
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
