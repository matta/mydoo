import type {DocHandleChangePayload} from '@automerge/automerge-repo';
import {useDocHandle} from '@automerge/automerge-repo-react-hooks';
import type React from 'react';
import {createContext, useContext, useEffect} from 'react';
import {Provider, useDispatch} from 'react-redux';
import {store as defaultStore} from '../store';
import {syncDoc} from '../store/slices/tasks-slice';
import {asAutomergeUrl, type DocumentHandle, type TunnelState} from '../types';

/**
 * Context to provide the Automerge document handle to hooks.
 */
const TaskLensContext = createContext<DocumentHandle | null>(null);

/**
 * Hook to access the current DocumentHandle from context.
 *
 * @returns The DocumentHandle provided by TaskLensProvider.
 * @throws Error if used outside of TaskLensProvider.
 */
export function useTaskLensDocId(): DocumentHandle {
  const docId = useContext(TaskLensContext);
  if (!docId) {
    throw new Error('useTaskLensDocId must be used within a TaskLensProvider');
  }
  return docId;
}

/**
 * Props for the TaskLensProvider component.
 *
 * @property docId - The Automerge document handle to synchronize with.
 * @property store - Optional Redux store to use (useful for tests).
 * @property children - React children to render within the provider.
 */
interface Props {
  docId: DocumentHandle;
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
function TaskLensSync({docId}: {docId: DocumentHandle}) {
  const dispatch = useDispatch<typeof defaultStore.dispatch>();
  // Cast to locally defined TypedDocHandle to ensure doc() and events are strictly typed
  const handle = useDocHandle<TunnelState>(asAutomergeUrl(docId));

  useEffect(() => {
    if (!handle) return;

    // Initial sync
    // We can safely call doc() because useDocHandle ensures the handle is ready when returned
    try {
      const doc = handle.doc();
      if (doc) {
        dispatch(syncDoc(doc as TunnelState));
      }
    } catch (e) {
      // Handle might not be ready in edge cases, though useDocHandle usually prevents this
      console.warn('Failed to get initial doc:', e);
    }

    const onDocChange = ({doc}: DocHandleChangePayload<TunnelState>) => {
      if (doc) {
        dispatch(syncDoc(doc));
      }
    };

    // Subscribe to future changes
    handle.on('change', onDocChange);

    return () => {
      handle.off('change', onDocChange);
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
 *   const docId = useDocument();
 *   return (
 *     <TaskLensProvider docId={docId}>
 *       <MyApp />
 *     </TaskLensProvider>
 *   );
 * }
 * ```
 */
export function TaskLensProvider({
  docId,
  store = defaultStore,
  children,
}: Props) {
  return (
    <TaskLensContext.Provider value={docId}>
      <Provider store={store}>
        <TaskLensSync docId={docId} />
        {children}
      </Provider>
    </TaskLensContext.Provider>
  );
}
