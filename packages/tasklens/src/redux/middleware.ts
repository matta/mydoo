import type { AnyDocumentId, DocHandle, Repo } from "@automerge/automerge-repo";
import type {
  Middleware,
  MiddlewareAPI,
  UnknownAction,
} from "@reduxjs/toolkit";
import { runReconciler } from "../domain/reconciler";
import { TunnelStateSchema } from "../persistence/schemas";
import type { TaskLensDispatch, TaskLensState } from "../store";
import { syncDoc } from "../store/slices/tasks-slice";
import type { TunnelState } from "../types/persistence";

/**
 * Extra argument passed to Redux thunks via thunkAPI.extra.
 * This provides type-safe access to the Automerge handle.
 */
export interface ThunkExtra {
  getHandle: () => DocHandle<TunnelState>;
}

/**
 * Result of creating the TaskLens middleware.
 * Includes the middleware itself and a factory for the thunk extra logic.
 */
export interface TaskLensMiddlewareResult {
  middleware: Middleware;
  getThunkExtra: () => ThunkExtra;
}

/**
 * Creates a Redux middleware and its associated thunk extra logic.
 *
 * This refactored version avoids global state by keeping the handle reference
 * inside the closure of the middleware factory. This allows multiple independent
 * stores to exist in the same process (critical for parallel tests).
 *
 * @param repo - The Automerge repo instance
 * @param docUrl - The document URL to sync
 * @returns An object containing the middleware and getExtra function
 */
export function createTaskLensMiddleware(
  repo: Repo,
  docUrl: string,
): TaskLensMiddlewareResult {
  let instanceHandle: DocHandle<TunnelState> | null = null;

  const middleware: Middleware = (storeApi) => {
    const store = storeApi as MiddlewareAPI<TaskLensDispatch, TaskLensState>;
    const handleOrPromise = repo.find<TunnelState>(docUrl as AnyDocumentId);

    Promise.resolve(handleOrPromise)
      .then((handle: DocHandle<TunnelState>) => {
        instanceHandle = handle;

        const processAndSync = (doc: TunnelState) => {
          const parseResult = TunnelStateSchema.safeParse(doc);
          if (parseResult.success) {
            store.dispatch(
              syncDoc({ proxyDoc: doc, parsedDoc: parseResult.data }),
            );
          } else {
            console.error(
              "[TaskLens] Doc failed validation during sync:",
              parseResult.error,
            );
          }
        };

        // Listen for changes
        handle.on("change", ({ doc }) => {
          processAndSync(doc);
        });

        // Initial Load
        const doc = handle.doc();
        if (doc) {
          const didMutate = runReconciler(handle);
          if (!didMutate) {
            processAndSync(doc);
          }
        }
      })
      .catch((err) => {
        // Log locally but don't crash the process. This is common in tests
        // where a mismatched repo/URL pair might be used by accident.
        console.warn(
          `[TaskLens] Middleware failed to find document: ${docUrl}`,
          err,
        );
      });

    return (next) => (action: unknown) => next(action as UnknownAction);
  };

  const getExtra = (): ThunkExtra => ({
    getHandle: () => {
      if (!instanceHandle) {
        throw new Error(
          "TaskLens handle not initialized or document not ready",
        );
      }
      return instanceHandle;
    },
  });

  return { middleware, getThunkExtra: getExtra };
}
