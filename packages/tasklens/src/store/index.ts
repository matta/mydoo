import {configureStore} from '@reduxjs/toolkit';
import tasksReducer from './slices/tasks-slice';

/**
 * Creates a fresh Redux store instance for TaskLens.
 *
 * @returns An configured Redux `EnhancedStore` holding the `RootState`.
 *   - **State**: Contains the `tasks` slice (`TasksState`) managing Automerge
 *     data.
 *   - **Middleware**: In Redux, middleware provides a third-party extension point
 *     between dispatching an action, and the moment it reaches the reducer.
 *     We use it here to:
 *       1. Handle async logic (Thunks).
 *       2. Enforce immutability guarantees.
 *       3. Check for non-serializable data (ignoring specific Automerge paths).
 *
 * We expose this as a factory function (rather than a singleton directly)
 * to allow creating independent store instances for:
 * 1. The main application (singleton), below.
 * 2. Automated tests (ensuring isolation between tests).
 */
export function createTaskLensStore() {
  return configureStore({
    reducer: {
      tasks: tasksReducer,
    },
    middleware: (getDefaultMiddleware) =>
      getDefaultMiddleware({
        serializableCheck: {
          ignoredActionPaths: ['payload.newDoc'],
          ignoredPaths: ['tasks.lastDoc'],
        },
      }),
  });
}

/**
 * The Redux store for TaskLens.
 *
 * This store provides a stable, memoized view of task data derived from
 * Automerge documents. It ensures referential stability for computed tasks,
 * preventing unnecessary React re-renders when sibling tasks are modified.
 *
 * The store is configured with serialization checks disabled for the
 * `lastDoc` path since TunnelState may contain Automerge-specific structures.
 */
export const taskLensStore = createTaskLensStore();

/**
 * The root state type for the TaskLens Redux store.
 * Use this type when accessing store state in selectors.
 */
export type TaskLensState = ReturnType<typeof taskLensStore.getState>;

/**
 * The dispatch type for the TaskLens Redux store.
 * Use this type for strongly-typed dispatch in components and thunks.
 */
export type TaskLensDispatch = typeof taskLensStore.dispatch;
