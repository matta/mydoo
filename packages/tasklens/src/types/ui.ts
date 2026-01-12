/**
 * Public type definitions for the UI layer.
 *
 * This module defines the types that are safe to expose to the
 * client / UI. It prevents implementation details (like internal
 * scoring factors) from leaking into the view layer.
 *
 * ARCHITECTURE NOTE:
 * These types are intentionally DUPLICATED from the persistence layer
 * to create a strict boundary. This ensures that changes to the
 * database schema (schema evolution) do not accidentally leak into
 * or break the UI contract.
 */

// ===========================================================================
// Branded IDs
// ===========================================================================
import type { PlaceID, TaskID } from "./persistence";

export type { PlaceID, TaskID };

/**
 * Common fields for a Task, stripped of index signatures.
 * Used to bridge strict UI types and loose persistence types.
 */
export interface TaskFields {
  readonly id: TaskID;
  readonly title: string;
  readonly notes: string;
  readonly parentId?: TaskID | undefined;
  readonly childTaskIds: TaskID[];
  readonly placeId?: PlaceID | undefined;
  readonly status: TaskStatus;
  readonly isSequential: boolean;
  readonly isAcknowledged: boolean;
  readonly lastCompletedAt?: number | undefined;
  readonly importance: number;
  readonly creditIncrement?: number | undefined;
  readonly credits: number;
  readonly desiredCredits: number;
  readonly creditsTimestamp: number;
  readonly priorityTimestamp: number;
  readonly schedule: Schedule;
  readonly repeatConfig?: RepeatConfig | undefined;

  // -- Effective Schedule (Computed / Ephemeral) --
  // These fields do NOT exist in the persisted database.
  // They are populated during the enrichment phase (domain layer)
  // and are required in the ComputedTask view model.
  readonly effectiveDueDate?: number | undefined;
  readonly effectiveLeadTime?: number | undefined;
  readonly effectiveScheduleSource?: "self" | "ancestor" | undefined;
}

/**
 * Common fields for a Schedule, stripped of index signatures.
 */
export interface ScheduleFields {
  type: "Once" | "Routinely" | "DueDate" | "Calendar";
  dueDate?: number | undefined;
  leadTime: number;
  lastDone?: number | undefined;
}

/**
 * Common fields for a RepeatConfig, stripped of index signatures.
 */
export interface RepeatConfigFields {
  frequency: "minutes" | "hours" | "daily" | "weekly" | "monthly" | "yearly";
  interval: number;
}

// ===========================================================================
// Constants & Enums
// ===========================================================================

/**
 * Default credit increment for tasks when not explicitly set.
 * This corresponds to "Standard Effort" (1 point) in the PRD.
 */
export const DEFAULT_CREDIT_INCREMENT = 0.5;

/**
 * Possible states for a Task.
 */
export const TaskStatus = {
  Pending: "Pending",
  Done: "Done",
} as const;

export type TaskStatus = (typeof TaskStatus)[keyof typeof TaskStatus];

// ===========================================================================
// Complex Value Objects
// ===========================================================================

/**
 * Scheduling information for a task.
 */
export interface Schedule {
  /** "Once" for one-time tasks, "Routinely" for repeating tasks. */
  type: "Once" | "Routinely" | "DueDate" | "Calendar";
  /** Unix timestamp (ms) when the task is due, or undefined if no deadline. */
  dueDate?: number | undefined;
  /** How far in advance (in ms) the task should appear before its due date. */
  leadTime: number;
  /** Timestamp of last completion (for Routinely). */
  lastDone?: number | undefined;
}

/**
 * Configuration for recurring tasks.
 */
export interface RepeatConfig {
  /** Frequency of recurrence */
  frequency: "minutes" | "hours" | "daily" | "weekly" | "monthly" | "yearly";
  /** Interval between occurrences (e.g., every 2 days) */
  interval: number;
}

/**
 * Filtering criteria for displaying tasks.
 */
export interface ViewFilter {
  includeClosed?: boolean | undefined;
  placeId?: PlaceID | "All" | undefined;
}

/**
 * Options for creating a new task, primarily for positioning.
 */
export type CreateTaskOptions =
  | { position: "start" }
  | { position: "end" }
  | { position: "after"; afterTaskId: TaskID };

/**
 * Fields allowed during task creation (UI Whitelist).
 *
 * This type excludes system-managed IDs and computed properties.
 */
export interface TaskCreateInput {
  title: string;
  parentId?: TaskID | undefined;
  placeId?: PlaceID | undefined;
  status?: TaskStatus;
  importance?: number;
  creditIncrement?: number | undefined;
  isSequential?: boolean;
  notes?: string;
  schedule?: Schedule;
  repeatConfig?: RepeatConfig | undefined;
  desiredCredits?: number;
  position?: "start" | "end" | "after";
  afterTaskId?: TaskID | undefined;
}

/**
 * Filtered properties for task creation UI.
 * Excludes control fields (title, parentId, position) that are usually handled separately.
 */
export type TaskCreateProps = Omit<
  TaskCreateInput,
  "title" | "parentId" | "position" | "afterTaskId"
>;

/**
 * Fields allowed during task update (UI Whitelist).
 *
 * This type strictly excludes primary IDs and all computed properties
 * to prevent accidental modification of internal state from the UI.
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
  desiredCredits?: number;
  schedule?: Schedule;
  repeatConfig?: RepeatConfig | undefined;
}

// ===========================================================================
// Task Objects
// ===========================================================================

/**
 * Public View Object (Read-Only).
 *
 * This is the object exposed to the Client / UI.
 * It contains a safe subset of persisted data plus computed helpers.
 * It specifically EXCLUDES internal scoring factors (priority, visibility, etc).
 */
export interface ComputedTask extends TaskFields {
  // --- Computed Helpers (Read-Only) ---

  /**
   * The timestamp-decayed value of the task's accumulated credits.
   * Represents "Recent Effort".
   */
  readonly effectiveCredits: number;

  /**
   * Indicates if this task is a parent node (has children).
   */
  readonly isContainer: boolean;

  /**
   * Indicates if the task is incomplete (Pending).
   */
  readonly isPending: boolean;

  /**
   * Indicates if the task has entered its active window.
   * True if current time is within Lead Time window matching the Schedule.
   */
  readonly isReady: boolean;

  /**
   * The effective due date (timestamp) derived from the task's schedule
   * or inherited from an ancestor.
   *
   * - Used for: Displaying "Due: <Date>" badges.
   * - Inherited via: Atomic Inheritance (Parent Date + Parent Lead Time).
   */
  readonly effectiveDueDate: number | undefined;

  /**
   * The effective lead time derived from the task's schedule or inherited.
   *
   * - Used for: Calculating urgency status.
   */
  readonly effectiveLeadTime: number | undefined;

  /**
   * Indicates where the effective schedule came from.
   * Useful for UI affordances (e.g. "inherited from parent").
   */
  readonly effectiveScheduleSource: "self" | "ancestor" | undefined;
}

/**
 * A Task with its children resolved into a tree structure.
 * Used for UI rendering where hierarchy traversal is needed.
 */
export interface TunnelNode extends ComputedTask {
  children: TunnelNode[];
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
