import type {AnyDocumentId} from '@automerge/automerge-repo';
/**
 * React integration for the Tunnel data model.
 *
 * This module provides a React hook (`useTunnel`) that connects to an Automerge
 * document and exposes typed operations for manipulating the task tree.
 *
 * Key concepts:
 * - **Automerge**: A library for building collaborative applications. It stores
 *   data in a CRDT (Conflict-free Replicated Data Type), enabling automatic
 *   synchronization across clients without a central server.
 * - **Document**: The Automerge "document" is the root data structure. Changes
 *   are made by calling `changeDoc((doc) => { ... })` with a callback that
 *   mutates the document proxy.
 * - **Zod validation**: At read time, we validate the document structure using
 *   Zod schemas to ensure the data conforms to our expected types. This is
 *   necessary because Automerge documents are untyped at runtime.
 */
import {useDocument} from '@automerge/automerge-repo-react-hooks';
import {useCallback, useMemo} from 'react';

import {
  type CreateTaskOptions,
  type DocumentHandle,
  type PersistedTunnelNode,
  type Task,
  type TaskID,
  TaskStatus,
  type TunnelNode,
  type TunnelState,
} from '../types';
import * as TunnelOps from './ops';
import {TunnelStateSchema} from './schemas';

/**
 * Return type of the `useTunnel` hook.
 *
 * This interface provides access to the task data and operations for
 * manipulating it. All operations are automatically synchronized to
 * other connected clients via Automerge.
 *
 * @property doc - The validated application state, or undefined if the document
 *                 hasn't loaded yet or failed validation.
 * @property tasks - The task tree as an array of root-level TunnelNodes.
 *                   Each node contains its children recursively. This is a read-only projection.
 * @property ops - Object containing operation functions to modify the state:
 *   - `add`: Create a new task with optional properties.
 *   - `update`: Modify properties of an existing task.
 *   - `move`: Relocate a task to a new parent or position.
 *   - `toggleDone`: Toggle a task's status between Done and Pending.
 *   - `delete`: Remove a task from the state.
 */
export interface TunnelHookResult {
  doc: TunnelState | undefined;
  ops: {
    add: (props: Partial<Task>, options?: CreateTaskOptions) => void;
    delete: (id: TaskID) => void;
    move: (
      id: TaskID,
      newParentId: TaskID | undefined,
      afterTaskId: TaskID | undefined,
    ) => void;
    toggleDone: (id: TaskID) => void;
    update: (id: TaskID, props: Partial<Task>) => void;
  };
  /**
   * Run a custom transaction on the document state.
   */
  change: (callback: (d: TunnelState) => void) => void;
  tasks: TunnelNode[];
}

/**
 * React hook for managing task data stored in an Automerge document.
 *
 * This hook connects to an Automerge document specified by a `DocumentHandle` and provides:
 * 1. Reactive access to the task tree (re-renders when the document changes).
 * 2. Type-safe operations for creating, updating, moving, and deleting tasks.
 *
 * The hook handles runtime validation of the document using Zod schemas.
 * If the document structure is invalid (corrupted or incompatible), `doc`
 * will be undefined and `tasks` will be an empty array.
 *
 * @param docId - The opaque DocumentHandle (erased Automerge URL).
 *                This is typically obtained from `useDocument()`.
 * @returns A `TunnelHookResult` containing the document, task tree, and operations.
 *
 * @example
 * ```typescript
 * function TaskList() {
 *   const docId = useDocument();
 *   const { tasks, ops } = useTunnel(docId);
 *
 *   return (
 *     <ul>
 *       {tasks.map(task => (
 *         <li key={task.id}>
 *           {task.title}
 *           <button onClick={() => ops.toggleDone(task.id)}>Toggle</button>
 *         </li>
 *       ))}
 *     </ul>
 *   );
 * }
 * ```
 *
 * @remarks
 * **React Hook Rules**: This function starts with "use" and follows React's
 * hook conventions. It must be called at the top level of a function component
 * and cannot be called conditionally.
 *
 * **Performance Note**: The `tasks` tree is rebuilt whenever the document
 * changes. For large lists, memoize child components.
 */

// Helper to hydrate PersistedTunnelNode to TunnelNode (Computed)
const enrichNode = (node: PersistedTunnelNode): TunnelNode => ({
  ...node,
  children: node.children.map(enrichNode),
  isContainer: node.childTaskIds.length > 0,
  isPending: node.status === TaskStatus.Pending,
  isReady: node.status === TaskStatus.Pending, // Simple approximation for Tree View
});

export function useTunnel(docId: DocumentHandle): TunnelHookResult {
  // Cast the opaque handle back to AnyDocumentId for the internal library
  const docUrl = docId as unknown as AnyDocumentId;
  const [doc, changeDoc] = useDocument(docUrl);

  const tasks = useMemo(() => {
    // Runtime validation for read
    const result = TunnelStateSchema.safeParse(doc);
    if (!result.success) return [];

    const rawTree = TunnelOps.getTaskTree(result.data);

    // Hydrate to Computed Nodes
    return rawTree.map(enrichNode);
  }, [doc]);

  // Wrap changeDoc to provide type-safe mutations on the Automerge proxy.
  // Note: We skip Zod validation inside the mutation callback for performance,
  // as we trust the repository contract and the document structure.
  const mutate = useCallback(
    (callback: (d: TunnelState) => void) => {
      changeDoc((d: unknown) => {
        // Type cast the internal Automerge proxy to our state type.
        callback(d as TunnelState);
      });
    },
    [changeDoc],
  );

  const add = useCallback(
    (props: Partial<Task>, options?: CreateTaskOptions) => {
      mutate(d => {
        TunnelOps.createTask(d, props, options);
      });
    },
    [mutate],
  );

  const update = useCallback(
    (id: TaskID, props: Partial<Task>) => {
      mutate(d => {
        TunnelOps.updateTask(d, id, props);
      });
    },
    [mutate],
  );

  const move = useCallback(
    (
      id: TaskID,
      newParentId: TaskID | undefined,
      afterTaskId: TaskID | undefined,
    ) => {
      mutate(d => {
        TunnelOps.moveTask(d, id, newParentId, afterTaskId);
      });
    },
    [mutate],
  );

  const toggleDone = useCallback(
    (id: TaskID) => {
      mutate(d => {
        const task = d.tasks[id];
        if (task) {
          task.status =
            task.status === TaskStatus.Done
              ? TaskStatus.Pending
              : TaskStatus.Done;
        }
      });
    },
    [mutate],
  );

  const remove = useCallback(
    (id: TaskID) => {
      mutate(d => {
        TunnelOps.deleteTask(d, id);
      });
    },
    [mutate],
  );

  // For the return value 'doc', we return the raw doc if it validates, or undefined
  const validDoc = useMemo(() => {
    const result = TunnelStateSchema.safeParse(doc);
    return result.success ? result.data : undefined;
  }, [doc]);

  return {
    doc: validDoc,
    tasks,
    change: mutate,
    ops: {
      add,
      update,
      move,
      toggleDone,
      delete: remove,
    },
  };
}
