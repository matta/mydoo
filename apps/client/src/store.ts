import type { Repo } from "@automerge/automerge-repo";
import { createTaskLensStore } from "@mydoo/tasklens";
import {
  type TypedUseSelectorHook,
  useDispatch,
  useSelector,
} from "react-redux";
import { repo } from "./lib/db";

/**
 * Creates the Redux store for the client application.
 *
 * This store delegates to the TaskLens store factory, which handles:
 * 1. State management via `tasksReducer`.
 * 2. Automerge sync middleware.
 * 3. Thunk extra arguments for document access.
 * 4. Serialization checks.
 *
 * @param docUrl - The Automerge document URL to sync with.
 * @param repoInstance - Optional repo instance (defaults to singleton).
 */
export function createClientStore(docUrl: string, repoInstance: Repo = repo) {
  return createTaskLensStore(repoInstance, docUrl);
}

// Infer types from the store factory
type ConfiguredStore = ReturnType<typeof createClientStore>;
type RootState = ReturnType<ConfiguredStore["getState"]>;
export type AppDispatch = ConfiguredStore["dispatch"];

// Use throughout your app instead of plain `useDispatch` and `useSelector`
export const useAppDispatch = () => useDispatch<AppDispatch>();
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector;
