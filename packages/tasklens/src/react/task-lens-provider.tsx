import type {AnyDocumentId} from '@automerge/automerge-repo';
import {useDocHandle} from '@automerge/automerge-repo-react-hooks';
import type React from 'react';
import {useEffect} from 'react';
import {Provider, useDispatch} from 'react-redux';
import type {AppDispatch} from '../store';
import {store} from '../store';
import {syncDoc} from '../store/slices/tasks-slice';
import type {DocumentHandle, TunnelState} from '../types';

/**
 * Props for the TaskLensProvider component.
 *
 * @property docId - The Automerge document handle to synchronize with.
 * @property children - React children to render within the provider.
 */
interface Props {
  docId: DocumentHandle;
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
  const dispatch = useDispatch<AppDispatch>();
  const handle = useDocHandle<TunnelState>(docId as unknown as AnyDocumentId);

  useEffect(() => {
    if (!handle) return;

    const syncFromHandle = () => {
      const doc = handle.docSync();
      if (doc) {
        dispatch(syncDoc(doc as TunnelState));
      }
    };

    // Initial sync
    syncFromHandle();

    // Subscribe to future changes
    handle.on('change', syncFromHandle);

    return () => {
      handle.off('change', syncFromHandle);
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
export function TaskLensProvider({docId, children}: Props) {
  return (
    <Provider store={store}>
      <TaskLensSync docId={docId} />
      {children}
    </Provider>
  );
}
