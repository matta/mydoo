/**
 * Internal type definitions for the Tunnel domain logic.
 *
 * This module defines types that are "enhanced" for the purpose of
 * task computations and algorithm processing. These types are NOT
 * exposed to the UI layer.
 */

import type { PersistedTask, PlaceID } from "./persistence";
import { DEFAULT_CREDIT_INCREMENT } from "./persistence";

export { DEFAULT_CREDIT_INCREMENT };

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
  mode?: "do-list" | "plan-outline";
  /**
   * Runtime context for algorithm calculations (time, location).
   */
  context?: Context;
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
