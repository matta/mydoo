/**
 * Core type definitions for the Tunnel data model (Persistence Layer).
 *
 * This module defines the structure of Tasks, Places, and the overall
 * application state (`TunnelState`) as they are stored in the database.
 */

import type { z } from "zod";

import type {
  PersistedTask,
  Place,
  PlaceIDSchema,
  RepeatConfig,
  Schedule,
  TaskIDSchema,
} from "../persistence/schemas";
import type { KnownKeysOnly } from "../utils/types";

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
 * Scheduling information for a task.
 *
 * Derived from ScheduleSchema in persistence/schemas.ts.
 * @see ScheduleSchema for the runtime validation schema.
 */
export type { Schedule };

/**
 * Configuration for recurring tasks.
 *
 * Derived from RepeatConfigSchema in persistence/schemas.ts.
 * @see RepeatConfigSchema for the runtime validation schema.
 */
export type { RepeatConfig };

/**
 * Time range string in HH:MM-HH:MM format (24h).
 */
export type OpenHoursRange = string;

/**
 * Defines the operating hours for a place.
 */
export interface OpenHours {
  /**
   * Operating mode: 'always_open', 'always_closed', or 'custom'.
   */
  mode: "always_open" | "always_closed" | "custom";
  /**
   * Weekly schedule mapping days to time ranges (required if mode is 'custom').
   */
  schedule?: {
    [day: string]: OpenHoursRange[];
  };
}

/**
 * A unit of work in the task management system (Persisted Record).
 *
 * This interface represents the raw data stored in the database (Automerge).
 * Unlike the legacy `Task` type, it does NOT contain computed properties.
 *
 * Derived from TaskSchema in persistence/schemas.ts.
 * @see TaskSchema for the runtime validation schema.
 */
export type { PersistedTask };

/**
 * A physical or virtual location where tasks can be performed.
 *
 * Derived from PlaceSchema in persistence/schemas.ts.
 * @see PlaceSchema for the runtime validation schema.
 */
export type { Place };

/**
 * The complete application state stored in the database.
 *
 * This is the root object that contains all tasks and places. It is stored
 * in Automerge (a CRDT library) for real-time synchronization between clients.
 *
 * @property nextPlaceId - Counter for generating unique place IDs.
 * @property nextTaskId - Counter for generating unique task IDs.
 * @property places - Map of place ID to Place object.
 * @property rootTaskIds - Ordered list of top-level task IDs (tasks with no parent).
 * @property tasks - Map of task ID to PersistedTask object. All tasks are stored flat here.
 *
 * The index signature is strictly required by @automerge/automerge types.
 * Without it, `Automerge.from<TunnelState>()` fails type checking.
 */
export interface TunnelState {
  [key: string]: unknown;
  places: Record<PlaceID, Place>;
  rootTaskIds: TaskID[];
  tasks: Record<TaskID, PersistedTask>;
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
  title?: string;
  parentId?: TaskID | undefined;
  placeId?: PlaceID | undefined;
  status?: TaskStatus;
  importance?: number;
  creditIncrement?: number | undefined;
  credits?: number;
  desiredCredits?: number;
  creditsTimestamp?: number;
  priorityTimestamp?: number;
  schedule?: KnownKeysOnly<Schedule>;
  repeatConfig?: KnownKeysOnly<RepeatConfig> | undefined;
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
  schedule?: KnownKeysOnly<Schedule>;
  repeatConfig?: KnownKeysOnly<RepeatConfig> | undefined;
  lastCompletedAt?: number | undefined;
}
