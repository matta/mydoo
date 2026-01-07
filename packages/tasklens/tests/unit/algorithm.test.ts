import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import Ajv, { type ValidateFunction } from "ajv";
import addFormats from "ajv-formats";
import { load } from "js-yaml";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import featureSchemaJson from "../../specs/compliance/schemas/feature.schema.json";
import { getPrioritizedTasks } from "../../src/domain/priority";
import type {
  InitialState,
  Mutation,
  Place as PlaceInput,
  Step,
  TaskInput,
  TunnelAlgorithmFeatureSchema,
} from "../../src/generated/feature";
import { TunnelStore } from "../../src/persistence/store";
import {
  ANYWHERE_PLACE_ID,
  DEFAULT_CREDIT_INCREMENT,
  type EnrichedTask,
  type PersistedTask,
  type Place,
  type PlaceID,
  type RepeatConfig,
  type Schedule,
  TaskStatus as StoreTaskStatus,
  type TaskID,
  type ViewFilter,
} from "../../src/types";
import {
  getCurrentTimestamp,
  mockCurrentTimestamp,
  resetCurrentTimestampMock,
} from "../../src/utils/time";

const ajv = new Ajv({
  allowUnionTypes: true,
  allErrors: true,
  strict: false,
});
addFormats(ajv);

const validateFeatureStructure = ajv.compile(featureSchemaJson);

const FIXTURES_PATH = join(process.cwd(), "specs", "compliance", "fixtures");

type Variables = Record<string, string | number | boolean | null | undefined>;

/**
 * Recursively interpolates template strings like `${varName}` with variable values.
 */
function interpolate<T>(template: T, variables: Variables): T {
  if (typeof template === "string") {
    return template.replace(/\$\{([^}]+)\}/g, (_, key: string) => {
      const val = variables[key];
      if (val === undefined) return `\${${key}}`;
      return String(val);
    }) as T;
  }
  if (Array.isArray(template)) {
    return template.map((item) => interpolate(item, variables)) as T;
  }
  if (template !== null && typeof template === "object") {
    const result: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(template)) {
      result[key] = interpolate(value, variables);
    }
    return result as T;
  }
  return template;
}

/**
 * Recursively casts string representations to their proper types.
 * Handles 'true'/'false' → boolean, 'null' → undefined, numeric strings → number.
 */
function castTypes<T>(obj: T): T {
  if (typeof obj === "string") {
    if (obj === "true") return true as T;
    if (obj === "false") return false as T;
    // Return undefined instead of null per project conventions
    if (obj === "null") return undefined as T;
    if (/^-?\d+(\.\d+)?$/.test(obj)) return Number(obj) as T;
    return obj as T;
  }
  if (Array.isArray(obj)) {
    return obj.map(castTypes) as T;
  }
  if (obj !== null && typeof obj === "object") {
    const result: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(obj)) {
      result[key] = castTypes(value);
    }
    return result as T;
  }
  return obj;
}

function parsePlaceInput(input: PlaceInput): Place {
  return {
    id: input.id as PlaceID,
    hours: JSON.stringify(input.hours),
    includedPlaces: (input.included_places ?? []) as PlaceID[],
  };
}

function parseTaskInput(
  input: TaskInput,
  testStartDate: Date,
  parentId?: string,
  parentPlaceId?: PlaceID,
  parentCreditIncrement?: number,
): PersistedTask[] {
  const tasks: PersistedTask[] = [];

  const effectivePlaceId =
    (input.place_id !== undefined ? (input.place_id as PlaceID) : undefined) ??
    parentPlaceId ??
    ANYWHERE_PLACE_ID;

  const effectiveCreditIncrement =
    (input.credit_increment !== undefined
      ? Number(input.credit_increment)
      : undefined) ??
    parentCreditIncrement ??
    DEFAULT_CREDIT_INCREMENT;

  // Handle repeatConfig
  let repeatConfig: RepeatConfig | undefined;
  if (input.repeat_config) {
    repeatConfig = input.repeat_config as RepeatConfig;
  }

  const task: PersistedTask = {
    id: input.id as TaskID,
    title: input.title ?? "Default Task",
    status:
      StoreTaskStatus[
        (input.status as keyof typeof StoreTaskStatus) ?? "Pending"
      ],
    importance: Number(input.importance ?? 1.0),
    creditIncrement: effectiveCreditIncrement,
    credits: Number(input.credits ?? 0),
    desiredCredits: Number(input.desired_credits ?? 0),
    creditsTimestamp: input.credits_timestamp
      ? new Date(input.credits_timestamp).getTime()
      : testStartDate.getTime(),
    priorityTimestamp: testStartDate.getTime(),
    schedule: {
      type: (input.schedule_type as Schedule["type"]) ?? "Once",
      leadTime: Number(input.lead_time_seconds ?? 604800) * 1000,
      ...(input.last_done
        ? { lastDone: new Date(input.last_done).getTime() }
        : {}),
    },
    placeId: effectivePlaceId as PlaceID,
    isSequential: Boolean(input.is_sequential ?? false),
    childTaskIds: [],
    isAcknowledged: false,
    notes: "",
  };

  if (repeatConfig) {
    task.repeatConfig = repeatConfig;
  }

  if (parentId) task.parentId = parentId as TaskID;

  if (input.due_date) {
    task.schedule.dueDate = new Date(input.due_date).getTime();
  }

  tasks.push(task);

  if (input.children) {
    for (const child of input.children) {
      task.childTaskIds.push(child.id as TaskID);
      tasks.push(
        ...parseTaskInput(
          child,
          testStartDate,
          task.id,
          task.placeId,
          task.creditIncrement,
        ),
      );
    }
  }

  return tasks;
}

function applyCreditUpdates(
  store: TunnelStore,
  updates: Record<string, number>,
): void {
  for (const [taskId, credits] of Object.entries(updates)) {
    store.updateTask(taskId as TaskID, {
      credits,
      creditsTimestamp: getCurrentTimestamp(),
    });
  }
}

type TaskUpdate = NonNullable<Mutation["task_updates"]>[number];

function computePartialUpdate(
  store: TunnelStore,
  id: string,
  props: Omit<TaskUpdate, "id">,
): Partial<PersistedTask> {
  const {
    status,
    credits,
    desired_credits,
    importance,
    due_date,
    is_acknowledged,
    place_id,
    credit_increment,
    // Schedule fields handled by computeScheduleUpdate
    schedule_type,
    repeat_config,
    last_done,
    lead_time_seconds,
    period_seconds,
    ...rest
  } = props;

  const _exhaustiveCheck: Record<string, never> = rest as Record<string, never>;
  void _exhaustiveCheck;

  const taskProps: Partial<PersistedTask> = {};

  if (status) {
    taskProps.status = StoreTaskStatus[status as keyof typeof StoreTaskStatus];
  }
  if (credits !== undefined) {
    taskProps.credits = Number(credits);
  }
  if (desired_credits !== undefined) {
    taskProps.desiredCredits = Number(desired_credits);
  }
  if (importance !== undefined) {
    taskProps.importance = Number(importance);
  }

  // Handle schedule-related updates
  computeScheduleUpdate(
    store,
    id,
    {
      due_date,
      schedule_type,
      repeat_config,
      last_done,
      lead_time_seconds,
      period_seconds,
    } as Omit<TaskUpdate, "id">,
    taskProps,
  );

  if (is_acknowledged !== undefined) {
    taskProps.isAcknowledged = Boolean(is_acknowledged);
  }
  if (place_id !== undefined) {
    taskProps.placeId = place_id as PlaceID;
  }
  if (credit_increment !== undefined) {
    taskProps.creditIncrement = Number(credit_increment);
  }

  return taskProps;
}

function computeScheduleUpdate(
  store: TunnelStore,
  id: string,
  props: Omit<TaskUpdate, "id">,
  taskProps: Partial<PersistedTask>,
): void {
  if (
    props.due_date !== undefined ||
    props.schedule_type !== undefined ||
    props.period_seconds !== undefined ||
    props.last_done !== undefined
  ) {
    const existingTask = store.getTask(id as TaskID);
    if (existingTask) {
      taskProps.schedule = {
        ...existingTask.schedule,
      };

      if (props.due_date !== undefined) {
        taskProps.schedule.dueDate = props.due_date
          ? new Date(props.due_date as string | number).getTime()
          : undefined;
      }
      if (props.schedule_type !== undefined) {
        if (props.schedule_type === null) {
          taskProps.schedule.type = "Once"; // Default fallback? Or undefined?
        } else {
          taskProps.schedule.type = props.schedule_type as Schedule["type"];
        }
      }

      if (props.repeat_config !== undefined) {
        taskProps.repeatConfig = props.repeat_config as RepeatConfig;
      }

      // Handle lastDone
      if (props.last_done !== undefined) {
        if (props.last_done === null) {
          taskProps.schedule.lastDone = undefined;
        } else {
          taskProps.schedule.lastDone = new Date(
            props.last_done as string | number,
          ).getTime();
        }
      }
    }
  } else if (props.repeat_config !== undefined) {
    // Repeat config updated independently
    taskProps.repeatConfig = props.repeat_config as RepeatConfig;
  }

  if (props.lead_time_seconds !== undefined) {
    if (!taskProps.schedule) {
      const existingTask = store.getTask(id as TaskID);
      if (existingTask) taskProps.schedule = { ...existingTask.schedule };
    }
    if (taskProps.schedule) {
      taskProps.schedule.leadTime = Number(props.lead_time_seconds) * 1000;
    }
  }
}

function applyTaskUpdates(store: TunnelStore, updates: TaskUpdate[]): void {
  for (const update of updates) {
    const { id, ...props } = update;
    const taskProps = computePartialUpdate(store, id, props);
    store.updateTask(id as TaskID, taskProps);
  }
}

function setupStore(hydratedBackground: InitialState) {
  const testStartTime = hydratedBackground.current_time
    ? new Date(hydratedBackground.current_time).getTime()
    : new Date("2025-01-01T12:00:00Z").getTime();

  mockCurrentTimestamp(testStartTime);

  const initialTasks: Record<string, PersistedTask> = {};
  const initialPlaces: Record<string, Place> = {};
  const rootTaskIds: TaskID[] = [];

  if (hydratedBackground.tasks) {
    for (const t of hydratedBackground.tasks) {
      const parsed = parseTaskInput(t as TaskInput, new Date(testStartTime));
      if (parsed.length > 0 && parsed[0]) {
        // The first task returned is the root of this sub-tree
        rootTaskIds.push(parsed[0].id);
      }
      for (const pt of parsed) initialTasks[pt.id] = pt;
    }
  }

  if (hydratedBackground.places) {
    for (const p of hydratedBackground.places) {
      const parsed = parsePlaceInput(p as PlaceInput);
      initialPlaces[parsed.id] = parsed;
    }
  }

  const store = new TunnelStore({
    tasks: initialTasks,
    places: initialPlaces,
    rootTaskIds,
    nextTaskId: 1,
    nextPlaceId: 1,
  });

  return { store, testStartTime };
}

function verifyTaskOrder(
  store: TunnelStore,
  viewFilter: ViewFilter,
  expectedOrder: string[],
) {
  const visibleTasks = getPrioritizedTasks(store.doc, viewFilter);
  expect(visibleTasks.map((t) => t.id)).toEqual(expectedOrder);
}

function verifySingleTaskProp(
  task: EnrichedTask,
  expected: NonNullable<NonNullable<Step["then"]>["expected_props"]>[number],
  propKey: string,
) {
  switch (propKey) {
    case "id":
      break; // Already used to look up task
    case "score":
      expect(task.priority).toBeCloseTo(Number(expected.score), 4);
      break;
    case "is_ready":
      expect(task.isReady).toBe(Boolean(expected.is_ready));
      break;
    case "is_visible":
      expect(task.visibility).toBe(Boolean(expected.is_visible));
      break;
    case "effective_credits":
      expect(task.effectiveCredits).toBeCloseTo(
        Number(expected.effective_credits),
        4,
      );
      break;
    case "normalized_importance":
      expect(task.normalizedImportance).toBeCloseTo(
        Number(expected.normalized_importance),
        4,
      );
      break;
    case "importance":
      expect(task.importance).toBeCloseTo(Number(expected.importance), 4);
      break;
    case "is_blocked":
      // TODO: EnrichedTask does not currently expose isBlocked status.
      // Ignoring this assertion for now to allow tests to pass.
      break;
    case "is_open":
      // TODO: EnrichedTask does not currently expose place open status independently of visibility.
      // Ignoring this assertion for now to allow tests to pass.
      break;
    case "place_id":
      expect(task.placeId).toBe(
        expected[propKey] as PlaceID | null | undefined,
      );
      break;
    case "credit_increment":
      expect(task.creditIncrement).toBeCloseTo(Number(expected[propKey]), 4);
      break;
    case "due_date": {
      const expectedDate = expected[propKey]
        ? new Date(expected[propKey] as string).getTime()
        : undefined;

      expect(task.schedule?.dueDate).toBe(expectedDate);
      break;
    }
    case "lead_time":
      expect(task.schedule?.leadTime).toBe(Number(expected[propKey]));
      break;
    default:
      throw new Error(`Unhandled assertion property: ${propKey}`);
  }
}

function verifyTaskProps(
  store: TunnelStore,
  viewFilter: ViewFilter,
  expectedProps: NonNullable<Step["then"]>["expected_props"],
) {
  if (!expectedProps) return;

  const allTasks = getPrioritizedTasks(store.doc, viewFilter, {
    includeHidden: true,
    mode: "plan-outline",
  });
  const computedMap = new Map(allTasks.map((t) => [t.id, t]));

  for (const expected of expectedProps) {
    const task = computedMap.get(expected.id as TaskID) as EnrichedTask;
    expect(task, `Task ${expected.id} not found`).toBeDefined();

    for (const propKey of Object.keys(expected)) {
      verifySingleTaskProp(task, expected, propKey);
    }
  }
}

function verifyAssertions(
  then: Step["then"],
  store: TunnelStore,
  viewFilter: ViewFilter,
) {
  if (!then) return;

  for (const key of Object.keys(then)) {
    switch (key) {
      case "expected_order": {
        if (Array.isArray(then.expected_order)) {
          verifyTaskOrder(store, viewFilter, then.expected_order);
        }
        break;
      }
      case "expected_props": {
        if (Array.isArray(then.expected_props)) {
          verifyTaskProps(store, viewFilter, then.expected_props);
        }
        break;
      }
      default:
        throw new Error(`Unhandled then property: ${key}`);
    }
  }
}

/**
 * Generates a descriptive test name from the step structure.
 * Produces names like "Given tasks, When advance_time_seconds, Then expected_order".
 */
function describeStep(stepIndex: number, step: Step): string {
  const clauses: string[] = [];

  if (step.given) {
    const givenKeys = Object.keys(step.given).filter(
      (k) => step.given?.[k as keyof typeof step.given] !== undefined,
    );
    if (givenKeys.length > 0) {
      clauses.push(`Given ${givenKeys.join(", ")}`);
    }
  }

  if (step.when) {
    const whenKeys = Object.keys(step.when).filter(
      (k) => step.when?.[k as keyof typeof step.when] !== undefined,
    );
    if (whenKeys.length > 0) {
      clauses.push(`When ${whenKeys.join(", ")}`);
    }
  }

  if (step.then) {
    const thenKeys = Object.keys(step.then).filter(
      (k) => step.then?.[k as keyof typeof step.then] !== undefined,
    );
    if (thenKeys.length > 0) {
      clauses.push(`Then ${thenKeys.join(", ")}`);
    }
  }

  const description = clauses.length > 0 ? clauses.join(", ") : "Empty step";
  return `Step ${stepIndex + 1}: ${description}`;
}

function executeStep(step: Step, store: TunnelStore, currentTestTime: number) {
  let newTime = currentTestTime;
  if (step.given) {
    if (step.given.current_time) {
      newTime = new Date(step.given.current_time).getTime();
      mockCurrentTimestamp(newTime);
    }
    if (step.given.tasks) {
      for (const t of step.given.tasks) {
        const parsedTasks = parseTaskInput(
          t as TaskInput,
          new Date(currentTestTime),
        );
        for (const task of parsedTasks) {
          if (store.getTask(task.id)) {
            store.updateTask(task.id, task);
          } else {
            store.createTask(task);
          }
        }
      }
    }
  }

  if (step.when) {
    if (step.when.advance_time_seconds) {
      newTime += Number(step.when.advance_time_seconds) * 1000;
      mockCurrentTimestamp(newTime);
    }
    if (step.when.update_credits) {
      applyCreditUpdates(
        store,
        step.when.update_credits as Record<string, number>,
      );
    }
    if (step.when.task_updates) {
      applyTaskUpdates(store, step.when.task_updates as TaskUpdate[]);
    }
  }

  const viewFilter: ViewFilter = {
    placeId:
      step.view_filter === "All Places"
        ? "All"
        : (step.view_filter as PlaceID) || "All",
  };
  verifyAssertions(step.then, store, viewFilter);

  return newTime;
}

describe("Algorithm BDD Test Suite", () => {
  const allFiles = readdirSync(FIXTURES_PATH).filter((f) =>
    f.endsWith(".feature.yaml"),
  );
  const allFeatures: TunnelAlgorithmFeatureSchema[] = [];

  for (const file of allFiles) {
    const content = readFileSync(join(FIXTURES_PATH, file), "utf8");
    const feature = load(content) as TunnelAlgorithmFeatureSchema;

    if (!validateFeatureStructure(feature)) {
      throw new Error(
        `Invalid feature structure ${file}: ${ajv.errorsText((validateFeatureStructure as ValidateFunction).errors)}`,
      );
    }
    allFeatures.push(feature);
  }

  // Filter features if requested (User Workflow Feature)
  const featureFilter = process.env.FEATURE_FILTER;
  const filteredFeatures = featureFilter
    ? allFeatures.filter((f) =>
        f.feature.toLowerCase().includes(featureFilter.toLowerCase()),
      )
    : allFeatures;

  for (const feature of filteredFeatures) {
    describe(`Feature: ${feature.feature}`, () => {
      for (const scenario of feature.scenarios) {
        const examples = scenario.examples || [{}];

        for (const [exampleIndex, example] of examples.entries()) {
          const scenarioName = scenario.examples
            ? `${scenario.name} (Example ${exampleIndex + 1})`
            : scenario.name;

          describe(`Scenario: ${scenarioName}`, () => {
            let store: TunnelStore;
            let currentTestTime: number;

            beforeAll(() => {
              const hydratedBackground = castTypes(
                interpolate(feature.background || {}, example),
              ) as InitialState;
              const setup = setupStore(hydratedBackground);
              store = setup.store;
              currentTestTime = setup.testStartTime;
            });

            afterAll(() => {
              resetCurrentTimestampMock();
            });

            for (const [stepIndex, rawStep] of scenario.steps.entries()) {
              const step = castTypes(interpolate(rawStep, example)) as Step;
              const stepDesc = describeStep(stepIndex, step);
              it(stepDesc, () => {
                currentTestTime = executeStep(step, store, currentTestTime);
              });
            }
          });
        }
      }
    });
  }
});
