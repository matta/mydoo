import type { Repo } from "@automerge/automerge-repo";
import { createTaskLensMiddleware, tasksReducer } from "@mydoo/tasklens";
import { configureStore } from "@reduxjs/toolkit";
import {
  type TypedUseSelectorHook,
  useDispatch,
  useSelector,
} from "react-redux";
import { repo } from "./lib/db";

/**
 * Creates the Redux store for the client application.
 *
 * This store:
 * 1. Uses the TaskLens `tasksReducer` to manage state.
 * 2. Injects the TaskLens middleware to sync with Automerge.
 * 3. Provides a thunk extra argument for type-safe document access.
 * 4. Uses the singleton `repo` instance by default, or an injected one for testing.
 *
 * @param docUrl - The Automerge document URL to sync with.
 * @param repoInstance - Optional repo instance (defaults to singleton).
 */
export function createClientStore(docUrl: string, repoInstance: Repo = repo) {
  const { middleware, getThunkExtra } = createTaskLensMiddleware(
    repoInstance,
    docUrl,
  );

  return configureStore({
    reducer: {
      tasks: tasksReducer,
    },
    middleware: (getDefaultMiddleware) =>
      getDefaultMiddleware({
        thunk: {
          extraArgument: getThunkExtra(),
        },
        serializableCheck: {
          ignoredActionPaths: ["payload.newDoc", "payload.proxyDoc"],
          ignoredPaths: ["tasks.lastDoc"],
        },
      }).concat(middleware),
  });
}

// Infer types from the store factory
type ConfiguredStore = ReturnType<typeof createClientStore>;
type RootState = ReturnType<ConfiguredStore["getState"]>;
export type AppDispatch = ConfiguredStore["dispatch"];

// Use throughout your app instead of plain `useDispatch` and `useSelector`
export const useAppDispatch = () => useDispatch<AppDispatch>();
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector;
