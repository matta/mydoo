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

import type {
  PersistedTask,
  Place,
  PlaceIDSchema,
  RepeatConfig,
  Schedule,
  TaskIDSchema,
} from './persistence/schemas';

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
  /**
   * Filter mode for determining which tasks to include.
   * - 'do-list' (default): Hides "Done" tasks unless they are unacknowledged.
   * - 'plan-outline': Shows all tasks regardless of status/acknowledgement.
   */
  mode?: 'do-list' | 'plan-outline';
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
 * Derived from ScheduleSchema in persistence/schemas.ts.
 * @see ScheduleSchema for the runtime validation schema.
 */
export type {Schedule};

/**
 * Configuration for recurring tasks.
 *
 * Derived from RepeatConfigSchema in persistence/schemas.ts.
 * @see RepeatConfigSchema for the runtime validation schema.
 */
export type {RepeatConfig};

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
 * Derived from TaskSchema in persistence/schemas.ts.
 * @see TaskSchema for the runtime validation schema.
 */
export type {PersistedTask};

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
  // =========================================================================
  // Internal Algorithm State
  // These fields are ephemeral scratchpad values used during the scoring passes.
  // They are NOT exposed to the UI via `ComputedTask`.
  // =========================================================================

  /**
   * The timestamp-decayed value of the task's accumulated credits.
   *
   * - Semantic Meaning: Represents "Recent Effort". Higher values mean the task
   *   (or its children) has been worked on recently, triggering the "Thermostat"
   *   feedback mechanism to lower priority and encourage switching context.
   * - "Do" List Impact: High effective credits reduce `feedbackFactor`, pushing
   *   the task down the list.
   * - "Plan" Outline Impact: None.
   */
  effectiveCredits: number;

  /**
   * A multiplier (0.0 - 1.0) derived from `effectiveCredits`.
   *
   * - Semantic Meaning: The "Boredom" or "Balancing" factor. As you work on a task,
   *   this drops from 1.0 towards 0.0, allowing other neglected tasks to rise.
   * - "Do" List Impact: Directly multiplies the final priority.
   * - "Plan" Outline Impact: None.
   */
  feedbackFactor: number;

  /**
   * A multiplier (0.0 - 1.0) based on how close the task is to its Due Date.
   *
   * - Semantic Meaning: "Urgency/Readiness".
   *   - 0.0: Task is too far in the future (Effective Start Date > Now).
   *   - 0.0 - 1.0: Ramps up as Due Date approaches (Lead Time window).
   *   - 1.0: Task is due or overdue.
   * - "Do" List Impact: Tasks with 0.0 are filtered out (isReady=false).
   *   Tasks with < 1.0 are deprioritized compared to fully urgent works.
   * - "Plan" Outline Impact: None directly, though affects `isReady`.
   */
  leadTimeFactor: number;

  /**
   * The calculated importance weight distributed from the root down to this node.
   *
   * - Semantic Meaning: "Relative Importance". Captures the user's manual ordering
   *   (`importance`) and tree structure. A child receives a fraction of its parent's
   *   importance.
   * - "Do" List Impact: The baseline score before feedback/urgency modulation.
   * - "Plan" Outline Impact: None.
   */
  normalizedImportance: number;

  /**
   * The final calculated score for the task (0.0 - 1.0).
   *
   * - Semantic Meaning: "What should I do next?".
   *   Formula: `normalizedImportance * leadTimeFactor * feedbackFactor * visibility`.
   * - "Do" List Impact: The primary sorting key.
   * - "Plan" Outline Impact: None.
   */
  priority: number;

  /**
   * Boolean flag indicating if the task is valid in the current CONTEXT.
   *
   * - Semantic Meaning: "Can I do this *here* and *now*?".
   *   - TRUE: Task matches the current Place filter AND the Place is currently "Open".
   *   - FALSE: Task is in a filtered-out Place or the Place is "Closed".
   *   - CRITICAL: Distinct from `isReady`. A task can be Visible (at office) but
   *     Not Ready (due next year).
   * - "Do" List Impact: Tasks with `visibility: false` have `priority: 0`.
   * - "Plan" Outline Impact: Used to filter the tree view if "Show Hidden" is off.
   */
  visibility: boolean;

  /**
   * The order of the task in the user's outline (0-based index).
   *
   * - Semantic Meaning: "User Definition Order".
   *   Used as a deterministic tie-breaker when priorities are equal.
   * - "Do" List Impact: Secondary sort key (ascending).
   */
  outlineIndex: number;

  // =========================================================================
  // Public Computed Properties
  // These fields are exposed to the UI via the `ComputedTask` interface.
  // =========================================================================

  /**
   * Helper indicating if this task is a parent node in the logic.
   *
   * - Semantic Meaning: "Is this a grouping mechanism?".
   *   True if the task has children.
   * - Exposed Via: `ComputedTask`
   */
  isContainer: boolean;

  /**
   * Helper indicating if the task is incomplete.
   *
   * - Semantic Meaning: Redundant alias for `status === 'Pending'`.
   *   This field is semantically identical to checking the status field directly.
   * - Exposed Via: `ComputedTask`
   */
  isPending: boolean;

  /**
   * Helper indicating if the task has entered its active window.
   *
   * - Semantic Meaning: "Has it started?".
   *   True if the current time is within the Lead Time window (or overdue).
   *   - 0.0: Task is scheduled for the future (Start Date > Now).
   *   - > 0.0: Task has started and is strictly available to work on.
   *   - CRITICAL: A task can be `isPending: true` (Not Done) but `isReady: false`
   *     (Not Started Yet).
   * - "Do" List Impact: The primary filter. Only `isReady: true` tasks appear.
   * - Exposed Via: `ComputedTask`
   */
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
  /**
   * The timestamp-decayed value of the task's accumulated credits.
   *
   * - Semantic Meaning: Represents "Recent Effort". Higher values mean the task
   *   (or its children) has been worked on recently.
   * - Exposed Via: `ComputedTask`
   */
  readonly effectiveCredits: number;

  /**
   * Indicates if this task is a parent node.
   *
   * - Semantic Meaning: "Is this a grouping mechanism?".
   *   True if the task has children.
   * - Exposed Via: `ComputedTask`
   */
  readonly isContainer: boolean;

  /**
   * Indicates if the task is incomplete.
   *
   * - Semantic Meaning: Redundant alias for `status === 'Pending'`.
   *   This field is semantically identical to checking the status field directly.
   * - Exposed Via: `ComputedTask`
   */
  readonly isPending: boolean;

  /**
   * Indicates if the task has entered its active window.
   *
   * - Semantic Meaning: "Has it started?".
   *   True if the current time is within the Lead Time window (or overdue).
   *   - 0.0: Task is scheduled for the future (Start Date > Now).
   *   - > 0.0: Task has started and is strictly available to work on.
   *   - CRITICAL: A task can be `isPending: true` (Not Done) but `isReady: false`
   *     (Not Started Yet).
   * - "Do" List Impact: The primary filter. Only `isReady: true` tasks appear.
   * - Exposed Via: `ComputedTask`
   */
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
 * Derived from PlaceSchema in persistence/schemas.ts.
 * @see PlaceSchema for the runtime validation schema.
 */
export type {Place};

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

/**
 * Data structure for an item in the Balance View.
 */
export interface BalanceItemData {
  id: TaskID;
  title: string;
  desiredCredits: number;
  effectiveCredits: number;
  targetPercent: number;
  actualPercent: number;
  isStarving: boolean;
}
