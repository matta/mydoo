/**
 * Runtime validation schemas using Zod.
 *
 * Zod is a TypeScript-first schema validation library. Unlike TypeScript's
 * compile-time type system, Zod validates data at runtime, which is essential
 * when working with data from external sources (like Automerge documents that
 * may have been modified by other clients or corrupted).
 *
 * These schemas are the **single source of truth** for persistence types.
 * TypeScript types are derived from these schemas using `z.infer<typeof Schema>`
 * to prevent drift between runtime validation and compile-time types.
 *
 * Usage: `TunnelStateSchema.safeParse(doc)` returns `{ success: true, data }`
 * if valid, or `{ success: false, error }` if not.
 *
 * @remarks
 * This follows the architecture decision documented in `ROLLING_CONTEXT.md`:
 * "Do not manually define TypeScript interfaces that mirror Zod schemas.
 * Derive the static type directly from the runtime schema using `z.infer`."
 */
import { z } from "zod";

/**
 * Helper to create a record schema that handles Automerge proxies.
 *
 * Automerge proxies contain internal symbols (e.g. Symbol(_am_meta)) which
 * can cause strict Zod record validation to fail if it inspects all own keys.
 * This helper preprocesses the object to only expose enumerable string keys,
 * effectively stripping the symbols before validation.
 *
 * @remarks Keys are always strings in our domain (TaskID, PlaceID are branded strings).
 */
function AutomergeRecord<
  // We must allow 'any' for the Definition and Input types to satisfy Zod's
  // internal variance checks (e.g. allowing z.string() which accepts unknown inputs).
  // biome-ignore lint/suspicious/noExplicitAny: internal variance checks
  K extends z.ZodType<string | number | symbol, any, any>,
  V extends z.ZodType,
>(keySchema: K, valueSchema: V) {
  return z.preprocess(
    (val) => {
      if (typeof val !== "object" || val === null) return val;
      const cleanObj: Record<string, unknown> = {};
      for (const k of Object.keys(val)) {
        cleanObj[k] = (val as Record<string, unknown>)[k];
      }
      return cleanObj;
    },
    z.record(keySchema, valueSchema),
  );
}

/**
 * Schema for validating a task ID.
 *
 * Uses Zod's `.brand()` to produce a branded type that matches `TaskID`
 * from types.ts. After validation, the output is typed as `TaskID`.
 */
export const TaskIDSchema = z.string().brand<"TaskID">();

/**
 * Schema for validating a place ID.
 *
 * Uses Zod's `.brand()` to produce a branded type that matches `PlaceID`.
 */
export const PlaceIDSchema = z.string().brand<"PlaceID">();

/**
 * Schema for validating a Schedule object.
 *
 * Scheduling information for a task.
 */
export const ScheduleSchema = z.looseObject({
  /** "Once" for one-time tasks, "Routinely" for repeating tasks. */
  type: z.enum(["Once", "Routinely", "DueDate", "Calendar"]),
  /** Unix timestamp (ms) when the task is due, or undefined if no deadline. */
  dueDate: z.number().optional(),
  /** How far in advance (in ms) the task should appear before its due date. */
  leadTime: z.number(),
  /** Timestamp of last completion (for Routinely). */
  lastDone: z.number().optional(),
});

/**
 * TypeScript type derived from ScheduleSchema.
 * This is the single source of truth - do not manually define a Schedule interface.
 */
export type Schedule = z.infer<typeof ScheduleSchema>;

/**
 * Schema for validating RepeatConfig.
 *
 * Configuration for recurring tasks.
 */
export const RepeatConfigSchema = z.looseObject({
  /** Frequency of recurrence */
  frequency: z.enum([
    "minutes",
    "hours",
    "daily",
    "weekly",
    "monthly",
    "yearly",
  ]),
  /** Interval between occurrences (e.g., every 2 days) */
  interval: z.number().min(1),
});

/**
 * TypeScript type derived from RepeatConfigSchema.
 * This is the single source of truth - do not manually define a RepeatConfig interface.
 */
export type RepeatConfig = z.infer<typeof RepeatConfigSchema>;

/**
 * Schema for validating a Task object (Persisted Record).
 *
 * This schema validates all required properties of a Task. Note that computed
 * properties (like `priority`, `visibility`) are not included because they
 * are calculated at runtime and not stored in the document.
 *
 * This represents the raw data stored in the database (Automerge).
 * Unlike computed task types, it does NOT contain computed properties.
 */
export const TaskSchema = z.looseObject({
  /** Unique identifier for this task. */
  id: TaskIDSchema,
  /** Human-readable name or description of the task. */
  title: z.string(),
  /** Markdown notes attached to the task. */
  notes: z.string().default(""),
  /** ID of the parent task, or undefined if this is a root task. */
  parentId: TaskIDSchema.optional(),
  /** Ordered list of child task IDs. */
  childTaskIds: z.array(TaskIDSchema),
  /** Location where this task should be done, or undefined to inherit from parent. */
  placeId: PlaceIDSchema.optional(),
  /** Current state: Pending, Done, or Deleted. */
  status: z.enum(["Pending", "Done"]),
  /** User-assigned priority from 0.0 (lowest) to 1.0 (highest). */
  importance: z.number().min(0).max(1),
  /** Points awarded when this task is completed. */
  creditIncrement: z.number().min(0).optional(),
  /** Accumulated points from completing this task and its children. */
  credits: z.number(),
  /**
   * Target allocation for this goal.
   * - Relevance: Root-level tasks only.
   * - Semantics: A weight representing the desired share of total effort.
   *   (Calculated as `desiredCredits / sum(all root credits)`).
   * - Default: 1.0.
   */
  desiredCredits: z.number().min(0),
  /** When credits were last modified (for decay calculations). */
  creditsTimestamp: z.number(),
  /** When priority was last recalculated. */
  priorityTimestamp: z.number(),
  /** Due date and recurrence information. */
  schedule: ScheduleSchema,
  /** Configuration for recurring tasks. */
  repeatConfig: RepeatConfigSchema.optional(),
  /** If true, children must be completed in order. */
  isSequential: z.boolean(),
  /** If true, completed task is hidden from "Do" view. */
  isAcknowledged: z.boolean().default(false),
  /** Timestamp when the task was last completed (for Routinely tasks). */
  lastCompletedAt: z.number().optional(),
});

/**
 * TypeScript type derived from TaskSchema.
 * This is the single source of truth - do not manually define a PersistedTask interface.
 *
 * Represents a unit of work in the task management system (Persisted Record).
 */
export type PersistedTask = z.infer<typeof TaskSchema>;

/**
 * Schema for validating a Place object.
 *
 * A physical or virtual location where tasks can be performed.
 */
export const PlaceSchema = z.looseObject({
  /** Unique identifier for this place. */
  id: PlaceIDSchema,
  /** Opening hours specification (serialized as string). */
  hours: z.string(),
  /**
   * Other place IDs that are "inside" this place.
   * For example, "Office" might include "Desk" and "Conference Room".
   */
  includedPlaces: z.array(PlaceIDSchema),
});

/**
 * TypeScript type derived from PlaceSchema.
 * This is the single source of truth - do not manually define a Place interface.
 */
export type Place = z.infer<typeof PlaceSchema>;

/**
 * Schema for validating the complete application state.
 *
 * This is the top-level schema used to validate Automerge documents before
 * they are used by the application. If validation fails, the document is
 * considered corrupted or incompatible.
 *
 * @see TunnelState in types.ts for the corresponding TypeScript interface.
 */
export const TunnelStateSchema = z.looseObject({
  rootTaskIds: z.array(TaskIDSchema),
  tasks: AutomergeRecord(TaskIDSchema, TaskSchema),
  places: AutomergeRecord(PlaceIDSchema, PlaceSchema),
});
