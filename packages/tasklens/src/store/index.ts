import {configureStore} from '@reduxjs/toolkit';
import tasksReducer from './slices/tasks-slice';

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
export const store = configureStore({
  reducer: {
    tasks: tasksReducer,
  },
  middleware: getDefaultMiddleware =>
    getDefaultMiddleware({
      serializableCheck: {
        // We might store TunnelState which could have non-serializable parts
        // if Automerge adds anything weird, though it should be POD.
        // For now let's be safe.
        ignoredActionPaths: ['payload.newDoc'],
        ignoredPaths: ['tasks.lastDoc'],
      },
    }),
});

/**
 * The root state type for the TaskLens Redux store.
 * Use this type when accessing store state in selectors.
 */
export type RootState = ReturnType<typeof store.getState>;

/**
 * The dispatch type for the TaskLens Redux store.
 * Use this type when dispatching actions or thunks.
 */
export type AppDispatch = typeof store.dispatch;
