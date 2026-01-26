/**
 * Core type definitions for the Tunnel data model (Persistence Layer).
 *
 * This module defines the structure of Tasks, Places, and the overall
 * application state (`TunnelState`) as they are stored in the database.
 */

import { ImmutableString } from "@automerge/automerge";
import type { z } from "zod";

import type {
  FrequencyType,
  PersistedTask,
  Place,
  PlaceIDSchema,
  RepeatConfig,
  Scalar,
  Schedule,
  ScheduleType,
  TaskIDSchema,
} from "../persistence/schemas";

export type {
  FrequencyType,
  PersistedTask,
  Place,
  RepeatConfig,
  Scalar,
  Schedule,
  ScheduleType,
};

export type OpenHoursRange = string;

export interface OpenHours {
  mode: "always_open" | "always_closed" | "custom";
  schedule?: {
    [day: string]: OpenHoursRange[];
  };
}

/** Factory for TaskStatus scalar */
export function toTaskStatusScalar(status: TaskStatus): Scalar<TaskStatus> {
  return new ImmutableString(String(status)) as Scalar<TaskStatus>;
}

/** Factory for ScheduleType scalar */
export function toScheduleTypeScalar(type: ScheduleType): Scalar<ScheduleType> {
  return new ImmutableString(String(type)) as Scalar<ScheduleType>;
}

/** Factory for Frequency scalar */
export function toFrequencyScalar(freq: FrequencyType): Scalar<FrequencyType> {
  return new ImmutableString(String(freq)) as Scalar<FrequencyType>;
}

/**
 * Safely extracts the value from a Scalar or primitive union.
 * Useful for reading Automerge values that might be proxied or raw.
 */
export function unwrapScalar<T extends string>(value: T | Scalar<T>): T {
  return typeof value === "string" ? value : value.val;
}

/**
 * Unique identifier for a Task.
 *
 * This is a "branded type" - a TypeScript pattern that prevents accidental
 * mixing of semantically different string values. At runtime, TaskID is just
 * a string with zero overhead. At compile time, TypeScript enforces that you
 * can't accidentally use a PlaceID where a TaskID is expected.
 *
 * The type is derived from the Zod schema to prevent drift between
 * runtime validation and compile-time types.
 *
 * To create a TaskID from a string, use explicit casting:
 * ```typescript
 * const id = String(numericId) as TaskID;
 * ```
 */
export type TaskID = z.infer<typeof TaskIDSchema>;

/**
 * Unique identifier for a Place.
 *
 * This is a "branded type" - see TaskID for explanation.
 * The type is derived from the Zod schema.
 */
export type PlaceID = z.infer<typeof PlaceIDSchema>;

/**
 * Reserved Place ID representing "any location".
 * Tasks assigned to this place are always visible regardless of filter.
 */
export const ANYWHERE_PLACE_ID = "Anywhere" as PlaceID;

/**
 * Default credit increment for tasks when not explicitly set.
 * This corresponds to "Standard Effort" (1 point) in the PRD.
 */
export const DEFAULT_CREDIT_INCREMENT = 0.5;

/**
 * Possible states for a Task.
 *
 * This is a "const object" pattern used in TypeScript to create an enum-like
 * structure that works at both compile time (for type checking) and runtime
 * (for value comparisons).
 *
 * - `Pending`: Task is not yet completed.
 * - `Done`: Task has been completed.
 */
export const TaskStatus = {
  Pending: "Pending",
  Done: "Done",
} as const;

/**
 * Union type of all possible TaskStatus values.
 * Derived from the TaskStatus const object for type-safe comparisons.
 */
export type TaskStatus = (typeof TaskStatus)[keyof typeof TaskStatus];

/**
 * A version of PersistedTask that allows `Scalar<T>` for enum fields.
 * This reflects the actual state in the Automerge document before sanitization.
 *
 * **Role in Program:**
 * - This type represents the **Mutable Proxy Object** inside an Automerge change function.
 * - It is providing the "Input/Output" for `ops.ts` and `store.ts`.
 * - **Scalars:** It accommodates Automerge's `Scalar` wrappers for primitive values (like strings that need conflict resolution).
 * - **Index Signature:** `[k: string]: unknown` explicitly allows "unknown" fields to persist, ensuring forward compatibility with future schema versions (data preservation).
 */
export type WritableTask = Pick<
  PersistedTask,
  | "id"
  | "title"
  | "notes"
  | "parentId"
  | "childTaskIds"
  | "placeId"
  | "importance"
  | "creditIncrement"
  | "credits"
  | "desiredCredits"
  | "creditsTimestamp"
  | "priorityTimestamp"
  | "isSequential"
  | "isAcknowledged"
  | "lastCompletedAt"
> & {
  // Allow booth primitive and Scalar forms for Automerge compatibility
  status: TaskStatus | Scalar<TaskStatus>;
  schedule: Omit<PersistedTask["schedule"], "type"> & {
    type: ScheduleType | Scalar<ScheduleType>;
  };
  repeatConfig?:
    | (Omit<Exclude<PersistedTask["repeatConfig"], undefined>, "frequency"> & {
        frequency: FrequencyType | Scalar<FrequencyType>;
      })
    | undefined;
  // Allow unknown fields without explicit catch-all in the type itself
  // to avoid index signature conflicts when nested.
  [k: string]: unknown;
};

/**
 * The complete application state stored in the database.
 *
 * This is the root object that contains all tasks and places. It is stored in
 * Automerge (a CRDT library) for real-time synchronization between clients.
 *
 * @property places - Map of place ID to Place object.
 * @property rootTaskIds - Ordered list of top-level task IDs (tasks with no
 * parent).
 * @property tasks - Map of task ID to PersistedTask object. All tasks are
 * stored flat here.
 *
 * The index signature is strictly required by @automerge/automerge types.
 * Without it, `Automerge.from<TunnelState>()` fails type checking.
 *
 * FIXME: Ideally this type is encapsulated in the persistence layer, and
 * application code should not need to know about it.
 */
export interface TunnelState {
  tasks: Record<TaskID, WritableTask>;
  rootTaskIds: TaskID[];
  places: Record<PlaceID, Place>;
  metadata?:
    | {
        automerge_url?: string | undefined;
      }
    | undefined;
  [key: string]: unknown;
}

export type CreateTaskOptions =
  | { position: "start" }
  | { position: "end" }
  | { position: "after"; afterTaskId: TaskID };

/**
 * Fields allowed during task creation.
 *
 * This includes all persisted fields to support creation from both
 * the UI and internal/migration paths.
 */
export interface TaskCreateInput {
  id?: TaskID;
  title: string;
  parentId?: TaskID | undefined;
  placeId?: PlaceID | undefined;
  status?: TaskStatus;
  importance?: number;
  creditIncrement?: number | undefined;
  credits?: number;
  desiredCredits?: number;
  creditsTimestamp?: number;
  priorityTimestamp?: number;
  schedule?: {
    type: ScheduleType;
    dueDate?: number | undefined;
    leadTime: number;
    lastDone?: number | undefined;
  };
  repeatConfig?:
    | {
        frequency: FrequencyType;
        interval: number;
      }
    | undefined;
  isSequential?: boolean;
  isAcknowledged?: boolean;
  notes?: string;
  lastCompletedAt?: number | undefined;
}

/**
 * Fields allowed during task update.
 *
 * This includes all persisted fields to support updates from both
 * the UI and internal/migration paths.
 */
export interface TaskUpdateInput {
  title?: string;
  notes?: string;
  parentId?: TaskID | undefined;
  placeId?: PlaceID | undefined;
  status?: TaskStatus;
  isSequential?: boolean;
  isAcknowledged?: boolean;
  importance?: number;
  creditIncrement?: number | undefined;
  credits?: number;
  desiredCredits?: number;
  creditsTimestamp?: number;
  priorityTimestamp?: number;
  schedule?: {
    type?: ScheduleType;
    dueDate?: number | undefined;
    leadTime?: number;
    lastDone?: number | undefined;
  };
  repeatConfig?:
    | {
        frequency?: FrequencyType;
        interval?: number;
      }
    | undefined;
  lastCompletedAt?: number | undefined;
}
