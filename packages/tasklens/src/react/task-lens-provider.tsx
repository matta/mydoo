import type {
  DocHandle,
  DocHandleChangePayload,
} from '@automerge/automerge-repo';
import type React from 'react';
import {useEffect} from 'react';
import {Provider, useDispatch} from 'react-redux';
import type {AppDispatch} from '../store';
import {store} from '../store';
import {syncDoc} from '../store/slices/tasks-slice';
import type {TunnelState} from '../types';

/**
 * Props for the TaskLensProvider component.
 *
 * @property docHandle - The Automerge document handle object to synchronize with.
 *                     If null or undefined, the Redux store will stop receiving updates
 *                     from the Automerge document ("disconnected" state).
 *                     Synchronization automatically resumes when a valid handle is provided.
 * @property children - React children to render within the provider.
 */
interface Props {
  docHandle: DocHandle<TunnelState> | null | undefined;
  children: React.ReactNode;
}

/**
 * Internal component to handle synchronization.
 * Must be child of Redux Provider to use useDispatch.
 *
 * Uses handle.on('change') to subscribe to document changes, ensuring
 * we capture all mutations including local changes.
 */
export function TaskLensSynchronizer({
  docHandle,
}: {
  docHandle: DocHandle<TunnelState> | null | undefined;
}) {
  const dispatch = useDispatch<AppDispatch>();

  useEffect(() => {
    if (!docHandle) return;

    // Use specific event payload type to access 'doc' directly
    const onHandleChange = ({doc}: DocHandleChangePayload<TunnelState>) => {
      dispatch(syncDoc(doc));
    };

    const initialSync = async () => {
      // doc() might be synchronous or return a Promise depending on version/state,
      // though types say synchronous. We handle both for robustness.
      // docSync() is deprecated.
      const docOrPromise = docHandle.doc();

      // Handle potential Promise check safely
      if (
        docOrPromise &&
        typeof (docOrPromise as unknown as Promise<TunnelState>).then ===
          'function'
      ) {
        const doc = await (docOrPromise as unknown as Promise<TunnelState>);
        if (doc) dispatch(syncDoc(doc));
      } else if (docOrPromise) {
        dispatch(syncDoc(docOrPromise as TunnelState));
      }
    };

    // Initial sync
    initialSync();

    // Subscribe to future changes
    docHandle.on('change', onHandleChange);

    return () => {
      docHandle.off('change', onHandleChange);
    };
  }, [docHandle, dispatch]);

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
 *   const handle = useDocHandle(docUrl);
 *   return (
 *     <TaskLensProvider docHandle={handle}>
 *       <MyApp />
 *     </TaskLensProvider>
 *   );
 * }
 * ```
 */
export function TaskLensProvider({docHandle, children}: Props) {
  return (
    <Provider store={store}>
      <TaskLensSynchronizer docHandle={docHandle} />
      {children}
    </Provider>
  );
}
