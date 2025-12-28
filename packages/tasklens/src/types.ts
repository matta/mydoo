/**
 * Core type definitions for the Tunnel data model.
 *
 * This module defines the structure of Tasks, Places, and the overall
 * application state (`TunnelState`).
 *
 * Key types:
 * - **Task**: A unit of work with properties like title, status, and priority.
 *   Tasks form a tree structure via `parentId` and `childTaskIds`.
 * - **TunnelNode**: A Task enriched with its resolved children. Used for
 *   rendering the task tree in the UI.
 * - **TunnelState**: The root state object stored in Automerge. Contains all
 *   tasks and places as key-value maps (Records).
 * - **Place**: A location or context where tasks can be performed (e.g., "Home").
 *
 * The `TaskStatus` pattern uses a const object with `as const` to create a
 * type-safe enum-like structure. This allows both runtime value checking
 * and compile-time type safety.
 */
import type {z} from 'zod';

import type {PlaceIDSchema, TaskIDSchema} from './persistence/schemas';

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
 * Opaque handle for a Tunnel Document.
 *
 * This type erases the underlying persistence implementation details (e.g. Automerge AnyDocumentId)
 * from the public API, allowing consumers to pass document references without importing persistence types.
 */
export type DocumentHandle = string & {__brand: 'DocumentHandle'};

/**
 * Reserved Place ID representing "any location".
 * Tasks assigned to this place are always visible regardless of filter.
 */
export const ANYWHERE_PLACE_ID = 'Anywhere' as PlaceID;

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
  Pending: 'Pending',
  Done: 'Done',
} as const;

/**
 * Union type of all possible TaskStatus values.
 * Derived from the TaskStatus const object for type-safe comparisons.
 */
export type TaskStatus = (typeof TaskStatus)[keyof typeof TaskStatus];

/**
 * Filtering criteria for displaying tasks.
 *
 * @property placeId - Optional. Show only tasks at this location.
 *                     Use "All" to show tasks at all locations.
 * @property includeClosed - Optional. If true, include completed/deleted tasks in results.
 */
export interface ViewFilter {
  includeClosed?: boolean;
  placeId?: PlaceID | 'All';
}

/**
 * Options to control which tasks are included in the prioritized output.
 */
export interface PriorityOptions {
  /** If true, include tasks with `visibility: false`. Defaults to false. */
  includeHidden?: boolean;
  /** If true, include acknowledged done tasks. Defaults to false. */
  includeDone?: boolean;
}

/**
 * Runtime context for algorithm calculations.
 *
 * @property currentTime - Current timestamp in milliseconds since Unix epoch.
 * @property currentPlaceId - Optional. The user's current location for filtering.
 */
export interface Context {
  currentPlaceId?: PlaceID;
  currentTime: number;
}

/**
 * Scheduling information for a task.
 *
 * @property type - "Once" for one-time tasks, "Recurring" for repeating tasks.
 * @property dueDate - Unix timestamp (ms) when the task is due, or undefined if no deadline.
 * @property leadTime - How far in advance (in ms) the task should appear before its due date.
 */
export interface Schedule {
  dueDate?: number | undefined;
  leadTime: number;
  type: 'Once' | 'Recurring';
}

/**
 * Configuration for recurring tasks.
 */
export interface RepeatConfig {
  /** Frequency of recurrence */
  frequency: 'daily' | 'weekly' | 'monthly' | 'yearly';
  /** Interval between occurrences (e.g., every 2 days) */
  interval: number;
}

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
  mode: 'always_open' | 'always_closed' | 'custom';
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
 * @property id - Unique identifier for this task.
 * @property title - Human-readable name or description of the task.
 * @property parentId - ID of the parent task, or undefined if this is a root task.
 * @property placeId - Location where this task should be done, or undefined to inherit from parent.
 * @property status - Current state: Pending, Done, or Deleted.
 * @property importance - User-assigned priority from 0.0 (lowest) to 1.0 (highest).
 * @property creditIncrement - Points awarded when this task is completed.
 * @property credits - Accumulated points from completing this task and its children.
 * @property desiredCredits - Target allocation for this task tree (used in prioritization).
 * @property creditsTimestamp - When credits were last modified (for decay calculations).
 * @property priorityTimestamp - When priority was last recalculated.
 * @property schedule - Due date and recurrence information.
 * @property isSequential - If true, children must be completed in order.
 * @property childTaskIds - Ordered list of child task IDs.
 * @property notes - Markdown notes attached to the task.
 * @property repeatConfig - Configuration for recurring tasks.
 * @property isAcknowledged - If true, completed task is hidden from "Do" view.
 */
export interface PersistedTask {
  childTaskIds: TaskID[];
  creditIncrement: number;
  credits: number;
  creditsTimestamp: number;
  desiredCredits: number;
  id: TaskID;
  importance: number;
  isSequential: boolean;
  parentId?: TaskID | undefined;
  placeId?: PlaceID | undefined;
  priorityTimestamp: number;
  schedule: Schedule;
  status: TaskStatus;
  title: string;
  notes: string;
  repeatConfig?: RepeatConfig | undefined;
  isAcknowledged: boolean;
}

/**
 * Internal Mutable Object for Algorithm Processing.
 *
 * This type is used exclusively within the `domain/` logic.
 * It extends `PersistedTask` with all the scratchpad fields needed by
 * the 7-pass prioritization algorithm.
 *
 * Performance Note: These objects are created via shallow clone for
 * extreme mutability performance in V8 hidden classes.
 */
export interface EnrichedTask extends PersistedTask {
  // --- Scores & Factors ---
  effectiveCredits: number;
  feedbackFactor: number;
  leadTimeFactor: number;
  normalizedImportance: number;
  priority: number;
  visibility: boolean;

  // --- State Helpers ---
  isContainer: boolean;
  isPending: boolean;
  isReady: boolean;
}

/**
 * Public View Object (Read-Only).
 *
 * This is the object exposed to the Client / UI.
 * It contains the persisted data plus a safe subset of computed helpers.
 * It specifically EXCLUDES internal scoring factors (priority, visibility, etc)
 * to prevent the UI from relying on implementation details.
 */
export interface ComputedTask extends PersistedTask {
  readonly isContainer: boolean;
  readonly isPending: boolean;
  readonly isReady: boolean;
}

/**
 * Legacy Alias for Client Compatibility.
 *
 * The client code expects a `Task` type. We point this to `ComputedTask`
 * so that components see the safe, read-only view of a task.
 *
 * TODO: This alias is temporary to ease migration. We should eventually:
 * 1. Allow the Client to handle `PersistedTask` for write operations (Editor).
 * 2. Explicitly use `ComputedTask` for read operations (Lists/Views).
 * 3. Fix the `pnpm typecheck` errors in `apps/client` where mocks/types are incorrect.
 */
export type Task = ComputedTask;

/**
 * Options for creating a new task, primarily for positioning.
 */
export type CreateTaskOptions =
  | {position: 'start'}
  | {position: 'end'}
  | {position: 'after'; afterTaskId: TaskID};

/**
 * A physical or virtual location where tasks can be performed.
 *
 * @property id - Unique identifier for this place.
 * @property hours - Opening hours specification (format TBD).
 * @property includedPlaces - Other place IDs that are "inside" this place.
 *                            For example, "Office" might include "Desk" and "Conference Room".
 */
export interface Place {
  hours: string;
  id: PlaceID;
  includedPlaces: PlaceID[];
}

/**
 * A Task with its children resolved into a tree structure.
 *
 * Used for UI rendering where the full tree hierarchy needs to be traversed.
 * Extends ComputedTask with a `children` array containing nested TunnelNodes.
 */
export interface TunnelNode extends ComputedTask {
  children: TunnelNode[];
}

/**
 * A Raw Persisted Task with its children resolved.
 * Used for recursive operations on the raw state (e.g. deletion).
 */
export interface PersistedTunnelNode extends PersistedTask {
  children: PersistedTunnelNode[];
}

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
