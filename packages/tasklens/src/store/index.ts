import type { Repo } from "@automerge/automerge-repo";
import {
  configureStore,
  type EnhancedStore,
  type Middleware,
  type ReducersMapObject,
  type ThunkDispatch,
  type UnknownAction,
} from "@reduxjs/toolkit";
import { createTaskLensMiddleware, type ThunkExtra } from "../redux/middleware";
import tasksReducer from "./slices/tasks-slice";

/**
 * Configuration object for integrating TaskLens into an existing Redux store.
 */
export interface TaskLensReduxConfig {
  /** The tasks slice reducer. Must be mounted at `state.tasks`. */
  reducer: typeof tasksReducer;
  /** The middleware that syncs Redux actions with Automerge. */
  middleware: Middleware;
  /**
   * The thunk extra argument providing access to the Automerge handle.
   * Pass this to `getDefaultMiddleware({ thunk: { extraArgument: ... } })`.
   */
  thunkExtra: ThunkExtra;
  /**
   * Recommended configuration for `serializableCheck` middleware.
   * Pass this to `getDefaultMiddleware({ serializableCheck: ... })`.
   */
  serializableCheckOptions: {
    ignoredActionPaths: string[];
    ignoredPaths: string[];
  };
}

/**
 * Generates the configuration needed to integrate TaskLens into a Redux store.
 * Use this if you are adding TaskLens to an existing application store.
 *
 * @param repo - The Automerge Repo instance.
 * @param docUrl - The Automerge document URL to sync with.
 * @returns The Redux configuration parts (reducer, middleware, settings).
 */
export function getTaskLensReduxConfig(
  repo: Repo,
  docUrl: string,
): TaskLensReduxConfig {
  const { middleware, getThunkExtra } = createTaskLensMiddleware(repo, docUrl);

  return {
    reducer: tasksReducer,
    middleware,
    thunkExtra: getThunkExtra(),
    serializableCheckOptions: {
      ignoredActionPaths: ["payload.newDoc", "payload.proxyDoc"],
      ignoredPaths: ["tasks.lastDoc"],
    },
  };
}

/**
 * Options for creating a TaskLens store.
 */
export interface CreateTaskLensStoreOptions {
  /** Additional reducers to merge into the store. */
  extraReducers?: ReducersMapObject;
  /** Additional middleware to prepend/append. */
  extraMiddleware?: Middleware[];
  /** Preloaded state for the store. */
  // biome-ignore lint/suspicious/noExplicitAny: Redux preloaded state is complex
  preloadedState?: any;
}

/**
 * Creates a fully configured Redux store for TaskLens.
 * Use this for creating independent store instances for apps or tests.
 *
 * @param repo - The Automerge Repo instance.
 * @param docUrl - The Automerge document URL to sync with.
 * @param options - Optional configuration for extra reducers/middleware.
 * @returns A configured Redux store.
 */
export function createTaskLensStore(
  repo: Repo,
  docUrl: string,
  options: CreateTaskLensStoreOptions = {},
): EnhancedStore<
  // biome-ignore lint/suspicious/noExplicitAny: State is extensible
  any,
  UnknownAction,
  // biome-ignore lint/suspicious/noExplicitAny: Enhancers are complex
  any
> & {
  // biome-ignore lint/suspicious/noExplicitAny: State is extensible
  dispatch: ThunkDispatch<any, ThunkExtra, UnknownAction>;
} {
  const config = getTaskLensReduxConfig(repo, docUrl);

  return configureStore({
    reducer: {
      tasks: config.reducer,
      ...options.extraReducers,
      // biome-ignore lint/suspicious/noExplicitAny: Fix TS overload resolution
    } as any,
    middleware: (getDefaultMiddleware) =>
      getDefaultMiddleware({
        thunk: {
          extraArgument: config.thunkExtra,
        },
        serializableCheck: config.serializableCheckOptions,
      })
        .concat(config.middleware)
        // biome-ignore lint/suspicious/noExplicitAny: Fix TS inference for middleware chain
        .concat(options.extraMiddleware || []) as any,
    preloadedState: options.preloadedState,
    // biome-ignore lint/suspicious/noExplicitAny: State is extensible
  }) as any;
}

/**
 * @deprecated Use `createTaskLensStore` instead.
 * This export exists only for backward compatibility during refactoring.
 */
export const taskLensStore = configureStore({
  reducer: { tasks: tasksReducer },
});

export type TaskLensState = ReturnType<typeof taskLensStore.getState>;
export type TaskLensDispatch = typeof taskLensStore.dispatch;
