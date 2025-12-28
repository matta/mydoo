import {configureStore} from '@reduxjs/toolkit';
import tasksReducer from './slices/tasks-slice';

/**
 * Creates a fresh Redux store instance for TaskLens.
 *
 * This factory function is useful for ensuring test isolation,
 * preventing state leakage between test cases.
 */
export function createStore() {
  return configureStore({
    reducer: {
      tasks: tasksReducer,
    },
    middleware: getDefaultMiddleware =>
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
export const store = createStore();

/**
 * The root state type for the TaskLens Redux store.
 * Use this type when accessing store state in selectors.
 */
export type RootState = ReturnType<typeof store.getState>;

/**
 * The store type for the TaskLens Redux store.
 */
export type AppStore = typeof store;

/**
 * The dispatch type for the TaskLens Redux store.
 * Use this type when dispatching actions or thunks.
 */
export type AppDispatch = typeof store.dispatch;
