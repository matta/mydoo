import type {
  AutomergeUrl,
  DocHandle,
  DocHandleChangePayload,
} from "@automerge/automerge-repo";
import { useDocHandle } from "@automerge/automerge-repo-react-hooks";
import type React from "react";
import { createContext, useContext, useEffect } from "react";
import { Provider, useDispatch } from "react-redux";
import { runReconciler } from "../domain/reconciler";
import { TunnelStateSchema } from "../persistence/schemas";
import type { TaskLensDispatch } from "../store";
import { taskLensStore as defaultStore } from "../store";
import { syncDoc } from "../store/slices/tasks-slice";
import type { TunnelState } from "../types/persistence";

/**
 * Initializes the document by running reconcilers and syncing to Redux.
 *
 * Flow:
 * 1. Get raw doc from Automerge (may contain legacy data)
 * 2. Run reconciler to migrate legacy schemas
 * 3. If reconciler mutated, abort (wait for change event)
 * 4. Parse with Zod to get validated TunnelState
 * 5. Dispatch to Redux
 */
function initDoc(handle: DocHandle<TunnelState>, dispatch: TaskLensDispatch) {
  // Step 1: Get raw doc (may have legacy data)
  const proxyDoc = handle.doc();
  if (!proxyDoc) return;

  // Step 2: Run reconciler on raw doc (mutates in place via handle.change)
  const didMutate = runReconciler(handle);
  if (didMutate) {
    // Abort. The mutation triggers a change event with fresh data.
    return;
  }

  // Step 3: Validate and convert to POJO.
  // After reconciliation, the doc should conform to TunnelStateSchema.
  // If it doesn't, that's a bug in the reconciler or a corrupted doc.
  const parseResult = TunnelStateSchema.safeParse(proxyDoc);
  if (!parseResult.success) {
    console.error(
      "[TaskLens] Doc failed validation after reconciliation:",
      parseResult.error,
    );
    return;
  }

  // Step 4: Dispatch the validated, strongly-typed doc
  dispatch(syncDoc({ proxyDoc: proxyDoc, parsedDoc: parseResult.data }));
}

/**
 * Context to provide the Automerge document handle to hooks.
 */
const TaskLensContext = createContext<AutomergeUrl | null>(null);

/**
 * Hook to access the current AutomergeUrl from context.
 *
 * @returns The AutomergeUrl provided by TaskLensProvider.
 * @throws Error if used outside of TaskLensProvider.
 */
export function useTaskLensDocUrl(): AutomergeUrl {
  const docUrl = useContext(TaskLensContext);
  if (!docUrl) {
    throw new Error("useTaskLensDocUrl must be used within a TaskLensProvider");
  }
  return docUrl;
}

/**
 * Props for the TaskLensProvider component.
 *
 * @property docUrl - The Automerge document URL to synchronize with.
 * @property store - Optional Redux store to use (useful for tests).
 * @property children - React children to render within the provider.
 */
interface Props {
  docUrl: AutomergeUrl;
  store?: typeof defaultStore;
  children: React.ReactNode;
}

/**
 * Internal component to handle synchronization.
 * Must be child of Redux Provider to use useDispatch.
 *
 * Uses handle.on('change') to subscribe to document changes, ensuring
 * we capture all mutations including local changes.
 */
function TaskLensSync({ docUrl }: { docUrl: AutomergeUrl }) {
  const dispatch = useDispatch<TaskLensDispatch>();
  // Cast to locally defined TypedDocHandle to ensure doc() and events are strictly typed
  const handle = useDocHandle<TunnelState>(docUrl);

  useEffect(() => {
    if (!handle) return;

    // Initial sync
    // We can safely call doc() because useDocHandle ensures the handle is ready when returned
    try {
      initDoc(handle, dispatch);
    } catch (e) {
      // Handle might not be ready in edge cases, though useDocHandle usually prevents this
      console.warn("Failed to get initial doc:", e);
    }

    const onDocChange = ({ doc }: DocHandleChangePayload<TunnelState>) => {
      if (doc) {
        initDoc(handle, dispatch);
      }
    };

    // Subscribe to future changes
    handle.on("change", onDocChange);

    return () => {
      handle.off("change", onDocChange);
    };
  }, [handle, dispatch]);

  return null;
}

/**
 * Provides the Redux-backed TaskLens store to the application.
 *
 * This component subscribes to Automerge document changes and synchronizes
 * them to the Redux store. It ensures referential stability for computed
 * tasks, preventing unnecessary React re-renders when possible.
 *
 * @example
 * ```tsx
 * function App() {
 *   const docUrl = useDocument();
 *   return (
 *     <TaskLensProvider docUrl={docUrl}>
 *       <MyApp />
 *     </TaskLensProvider>
 *   );
 * }
 * ```
 */
export function TaskLensProvider({
  docUrl,
  store = defaultStore,
  children,
}: Props) {
  return (
    <TaskLensContext.Provider value={docUrl}>
      <Provider store={store}>
        <TaskLensSync docUrl={docUrl} />
        {children}
      </Provider>
    </TaskLensContext.Provider>
  );
}
