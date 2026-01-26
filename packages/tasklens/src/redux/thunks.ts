import { createAsyncThunk } from "@reduxjs/toolkit";
import {
  createTask as createTaskOp,
  deleteTask as deleteTaskOp,
  moveTask as moveTaskOp,
  updateTask as updateTaskOp,
} from "../persistence/ops";
import type { TaskLensState } from "../store";
import type { TaskID, TunnelState } from "../types/persistence";
import {
  type TaskCreateInput,
  TaskStatus,
  type TaskUpdateInput,
  unwrapScalar,
} from "../types/ui";
import type { ThunkExtra } from "./middleware";

/**
 * Pre-typed createAsyncThunk with TaskLens state and extra argument.
 *
 * Using RTK's withTypes() helper eliminates manual type casts at each thunk
 * call site. The extra argument is strongly typed as ThunkExtra.
 */
const createTaskLensThunk = createAsyncThunk.withTypes<{
  state: TaskLensState;
  extra: ThunkExtra;
}>();

/**
 * Helper to access the handle and execute a change.
 * Now receives properly typed ThunkExtra from the typed thunk.
 */
const withHandle = (extra: ThunkExtra, cb: (doc: TunnelState) => void) => {
  const handle = extra.getHandle();
  handle.change((doc) => cb(doc));
};

export const createTask = createTaskLensThunk(
  "tasks/create",
  async (input: TaskCreateInput & { id?: TaskID }, { extra }) => {
    // Allows client to provide ID (for optimistic/sync UI), or generates one.
    const id = (input.id ?? crypto.randomUUID()) as TaskID;
    const { position, afterTaskId, ...props } = input;

    withHandle(extra, (doc) => {
      createTaskOp(
        doc,
        { ...props, id },
        input.position === "after" && input.afterTaskId
          ? { position: "after", afterTaskId: input.afterTaskId }
          : input.position === "start" || input.position === "end"
            ? { position: input.position }
            : undefined,
      );
    });

    return id;
  },
);

export const updateTask = createTaskLensThunk(
  "tasks/update",
  async (
    { id, updates }: { id: TaskID; updates: TaskUpdateInput },
    { extra },
  ) => {
    withHandle(extra, (doc) => {
      updateTaskOp(doc, id, updates);
    });
    return id;
  },
);

export const deleteTask = createTaskLensThunk(
  "tasks/delete",
  async (id: TaskID, { extra }) => {
    withHandle(extra, (doc) => {
      deleteTaskOp(doc, id);
    });
    return id;
  },
);

export const moveTask = createTaskLensThunk(
  "tasks/move",
  async (
    {
      id,
      newParentId,
      afterTaskId,
    }: {
      id: TaskID;
      newParentId: TaskID | undefined;
      afterTaskId: TaskID | undefined;
    },
    { extra },
  ) => {
    withHandle(extra, (doc) => {
      moveTaskOp(doc, id, newParentId, afterTaskId);
    });
    return id;
  },
);

export const acknowledgeAllDoneTasks = createTaskLensThunk(
  "tasks/acknowledgeAllDone",
  async (_, { extra }) => {
    withHandle(extra, (doc) => {
      const tasks = Object.values(doc.tasks);
      for (const task of tasks) {
        if (
          unwrapScalar(task.status) === TaskStatus.Done &&
          !task.isAcknowledged
        ) {
          task.isAcknowledged = true;
        }
      }
    });
  },
);
