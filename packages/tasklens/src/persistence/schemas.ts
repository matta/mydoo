/**
 * Runtime validation schemas using Zod.
 *
 * Zod is a TypeScript-first schema validation library. Unlike TypeScript's
 * compile-time type system, Zod validates data at runtime, which is essential
 * when working with data from external sources (like Automerge documents that
 * may have been modified by other clients or corrupted).
 *
 * These schemas mirror the TypeScript interfaces in `types.ts` and are used to:
 * 1. Validate that an Automerge document has the expected structure before use.
 * 2. Provide type narrowing: after successful validation, TypeScript knows
 *    the data conforms to the schema's type.
 *
 * Usage: `TunnelStateSchema.safeParse(doc)` returns `{ success: true, data }`
 * if valid, or `{ success: false, error }` if not.
 *
 * @remarks
 * **Schema Drift Warning**: These schemas are manually synchronized with the
 * TypeScript interfaces in `types.ts`. If the interfaces change, these schemas
 * must be updated manually to match.
 *
 * **Options to prevent drift**:
 * 1. **Zod as source of truth (RECOMMENDED)**: Define schemas here and use
 *    `z.infer<typeof Schema>` to derive TypeScript types. This eliminates
 *    `types.ts` duplication but requires refactoring all type imports.
 * 2. **ts-to-zod**: Auto-generate this file from `types.ts` at build time.
 *    Keeps `types.ts` as source of truth but adds a build step.
 * 3. **Manual sync (current)**: Keep both files and update manually. Simplest
 *    but prone to drift.
 */
import {z} from 'zod';

/**
 * Helper to create a record schema that handles Automerge proxies.
 *
 * Automerge proxies contain internal symbols (e.g. Symbol(_am_meta)) which
 * can cause strict Zod record validation to fail if it inspects all own keys.
 * This helper preprocesses the object to only expose enumerable string keys,
 * effectively stripping the symbols before validation.
 */
/* eslint-disable @typescript-eslint/no-explicit-any */
function AutomergeRecord<
  K extends z.ZodType<any, any, any>,
  V extends z.ZodType<any, any, any>,
>(keySchema: K, valueSchema: V) {
  /* eslint-enable @typescript-eslint/no-explicit-any */
  return z.preprocess(
    val => {
      if (typeof val !== 'object' || val === null) return val;
      // Create a shallow copy using only enumerable string keys.
      // This strips out Automerge's internal symbols.
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
export const TaskIDSchema = z.string().brand<'TaskID'>();

/**
 * Schema for validating a place ID.
 *
 * Uses Zod's `.brand()` to produce a branded type that matches `PlaceID`.
 */
export const PlaceIDSchema = z.string().brand<'PlaceID'>();

/**
 * Schema for validating a Schedule object.
 *
 * @see Schedule in types.ts for the corresponding TypeScript interface.
 */
export const ScheduleSchema = z.object({
  type: z.enum(['Once', 'Recurring']),
  dueDate: z.number().optional(),
  leadTime: z.number(),
});

/**
 * Schema for validating a Task object.
 *
 * This schema validates all required properties of a Task. Note that computed
 * properties (like `priority`, `visibility`) are not included because they
 * are calculated at runtime and not stored in the document.
 *
 * @see Task in types.ts for the corresponding TypeScript interface.
 */
export const TaskSchema = z.object({
  id: TaskIDSchema,
  title: z.string(),
  parentId: TaskIDSchema.optional(),
  childTaskIds: z.array(TaskIDSchema),
  placeId: PlaceIDSchema.optional(),
  status: z.enum(['Pending', 'Done', 'Deleted']),
  importance: z.number().min(0).max(1),
  creditIncrement: z.number().min(0),
  credits: z.number(),
  desiredCredits: z.number().min(0),
  creditsTimestamp: z.number(),
  priorityTimestamp: z.number(),
  schedule: ScheduleSchema,
  isSequential: z.boolean(),
});

/**
 * Schema for validating the complete application state.
 *
 * This is the top-level schema used to validate Automerge documents before
 * they are used by the application. If validation fails, the document is
 * considered corrupted or incompatible.
 *
 * @see TunnelState in types.ts for the corresponding TypeScript interface.
 *
 * @remarks
 * The `places` field uses `z.any()` as a placeholder. A proper PlaceSchema
 * should be defined if Place validation is needed.
 */
export const TunnelStateSchema = z.object({
  nextTaskId: z.number(),
  nextPlaceId: z.number(),
  rootTaskIds: z.array(TaskIDSchema),
  tasks: AutomergeRecord(TaskIDSchema, TaskSchema),
  places: AutomergeRecord(PlaceIDSchema, z.any()),
});
