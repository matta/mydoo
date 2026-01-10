import { useDocHandle } from "@automerge/automerge-repo-react-hooks";
import { useCallback } from "react";
import {
  createTask as createTaskOp,
  deleteTask as deleteTaskOp,
  moveTask as moveTaskOp,
  updateTask as updateTaskOp,
} from "../../persistence/ops";
import type { TaskID, TunnelState } from "../../types/persistence";
import {
  type TaskCreateInput,
  TaskStatus,
  type TaskUpdateInput,
} from "../../types/ui";
import { useTaskLensDocUrl } from "../task-lens-provider";

/**
 * Hook to perform mutations on the TaskLens document.
 *
 * This hook provides type-safe methods for creating, updating,
 * and moving tasks. It automatically handles the Automerge
 * transaction lifecycle.
 */
export function useTaskActions() {
  const docUrl = useTaskLensDocUrl();
  // biome-ignore lint/suspicious/noExplicitAny: internal handle type erasure
  const handle = useDocHandle<TunnelState>(docUrl as any);

  const mutate = useCallback(
    (callback: (d: TunnelState) => void) => {
      if (!handle) return;
      handle.change((d) => {
        callback(d as TunnelState);
      });
    },
    [handle],
  );

  const createTask = useCallback(
    (input: TaskCreateInput): TaskID => {
      const id = crypto.randomUUID() as TaskID;
      const { position, afterTaskId, ...props } = input;
      mutate((d) => {
        createTaskOp(
          d,
          {
            ...props,
            id,
          },
          input.position === "after" && input.afterTaskId
            ? { position: "after", afterTaskId: input.afterTaskId }
            : input.position === "start" || input.position === "end"
              ? { position: input.position }
              : undefined,
        );
      });
      return id;
    },
    [mutate],
  );

  const updateTask = useCallback(
    (id: TaskID, updates: TaskUpdateInput) => {
      mutate((d) => {
        updateTaskOp(d, id, updates);
      });
    },
    [mutate],
  );

  const deleteTask = useCallback(
    (id: TaskID) => {
      mutate((d) => {
        deleteTaskOp(d, id);
      });
    },
    [mutate],
  );

  const moveTask = useCallback(
    (
      id: TaskID,
      newParentId: TaskID | undefined,
      afterTaskId: TaskID | undefined,
    ) => {
      mutate((d) => {
        moveTaskOp(d, id, newParentId, afterTaskId);
      });
    },
    [mutate],
  );

  const acknowledgeAllDoneTasks = useCallback(() => {
    mutate((doc) => {
      // Keys are Persistence IDs
      const tasks = Object.values(doc.tasks);
      for (const task of tasks) {
        if (task && task.status === TaskStatus.Done && !task.isAcknowledged) {
          task.isAcknowledged = true;
        }
      }
    });
  }, [mutate]);

  return {
    createTask,
    updateTask,
    deleteTask,
    moveTask,
    acknowledgeAllDoneTasks,
  };
}
