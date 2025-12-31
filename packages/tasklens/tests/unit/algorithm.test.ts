import {readdirSync, readFileSync} from 'node:fs';
import {basename, join} from 'node:path';
import Ajv from 'ajv';
import addFormats from 'ajv-formats';
import {load} from 'js-yaml';
import {afterAll, beforeAll, describe, expect, it} from 'vitest';

import type {
  Place as PlaceInput,
  TaskInput,
  TunnelAlgorithmTestCaseSchema,
} from '../../specs/compliance/schemas/test_case';
import schemaJson from '../../specs/compliance/schemas/test_case.schema.json';
import {TunnelStore} from '../../src/persistence/store';
import {
  type EnrichedTask,
  type PersistedTask,
  type Place,
  type PlaceID,
  TaskStatus as StoreTaskStatus,
  type TaskID,
  TaskStatus,
  type ViewFilter,
} from '../../src/types';
import {
  daysToMilliseconds,
  getCurrentTimestamp,
  mockCurrentTimestamp,
  resetCurrentTimestampMock,
} from '../../src/utils/time';

const ajv = new Ajv();
addFormats(ajv);
const validate = ajv.compile(schemaJson);

const FIXTURES_PATH = join(process.cwd(), 'specs', 'compliance', 'fixtures');

// Explicit list of fixtures to ensure no unexpected files are processed (or missed).
const EXPECTED_FIXTURES = [
  'balancing.yaml',
  'boost-importance.yaml',
  'boost-lead-time.yaml',
  'complex-mutation.yaml',
  'decay.yaml',
  'lead-time-edge-cases.yaml',
  'lead-time.yaml',
  'min-threshold.yaml',
  'repro-stale-leadtime.yaml',
  'root-importance.yaml',
  'sequential-flow.yaml',
  'sorting.yaml',
  'thermostat.yaml',
  'tree-order-id-conflict.yaml',
  'tree-order.yaml',
  'visibility-place-filtering.yaml',
  'weight.yaml',
  'zero-feedback.yaml',
].sort();

// Validate that the directory matches exactly the expected list
const actualFixtures = readdirSync(FIXTURES_PATH)
  .filter((f: string) => f.endsWith('.yaml'))
  .sort();

// Simple equality check
const missing = EXPECTED_FIXTURES.filter(f => !actualFixtures.includes(f));
const extras = actualFixtures.filter(f => !EXPECTED_FIXTURES.includes(f));

if (missing.length > 0 || extras.length > 0) {
  throw new Error(
    `Fixture mismatch!\nMissing: ${missing.join(', ')}\nExtras: ${extras.join(
      ', ',
    )}`,
  );
}

const fixtureFiles = EXPECTED_FIXTURES;

function parsePlaceInput(input: PlaceInput): Place {
  return {
    id: input.id as PlaceID,
    hours: JSON.stringify(input.hours),
    includedPlaces: (input.included_places ?? []) as PlaceID[],
  };
}

// Helper to recursively parse TaskInput to PersistedTask
function parseTaskInput(
  input: TaskInput,
  testStartDate: Date,
  parentId?: string,
): PersistedTask[] {
  const tasks: PersistedTask[] = [];

  const statusMap: Record<string, StoreTaskStatus> = {
    Pending: StoreTaskStatus.Pending,
    Done: StoreTaskStatus.Done,
  };

  const task: PersistedTask = {
    id: input.id as TaskID,
    title: input.title ?? 'Default Task',
    status:
      (input.status ? statusMap[input.status] : undefined) ??
      TaskStatus.Pending,
    importance: input.importance ?? 1.0,
    creditIncrement: 1.0, // Default
    credits: input.credits ?? 0.0,
    desiredCredits: input.desired_credits ?? 0.0,
    creditsTimestamp:
      input.credits_timestamp && input.credits_timestamp !== '0'
        ? new Date(input.credits_timestamp).getTime()
        : testStartDate.getTime(),
    priorityTimestamp: testStartDate.getTime(),
    schedule: {
      type: 'Once', // Default
      leadTime: (input.lead_time_seconds ?? 604800) * 1000, // Convert seconds to ms
    },
    isSequential: input.is_sequential ?? false,
    childTaskIds: [],
    isAcknowledged: false, // Default for tests
    notes: '',
  };

  if (parentId) task.parentId = parentId as TaskID;
  if (input.place_id) task.placeId = input.place_id as PlaceID;

  const calculatedDueDate =
    typeof input.due_date === 'string'
      ? input.due_date
        ? new Date(input.due_date).getTime()
        : undefined
      : typeof input.due_date === 'number'
        ? new Date(
            testStartDate.getTime() +
              daysToMilliseconds(input.due_date as number),
          ).getTime()
        : undefined;

  if (calculatedDueDate !== undefined) {
    task.schedule.dueDate = calculatedDueDate;
  }

  // Handle properties potentially missing from strict TaskInput type but present in YAML
  const extendedInput = input as TaskInput & {credit_increment?: number};
  if (extendedInput.credit_increment !== undefined) {
    task.creditIncrement = extendedInput.credit_increment;
  }

  tasks.push(task);

  if (input.children) {
    for (const child of input.children) {
      task.childTaskIds.push(child.id as TaskID);
      tasks.push(...parseTaskInput(child, testStartDate, task.id));
    }
  }

  return tasks;
}

/**
 * Applies time advancement mutation to the store.
 */
function applyTimeAdvancement(advanceSeconds: number): void {
  const newTime = getCurrentTimestamp() + advanceSeconds * 1000;
  mockCurrentTimestamp(newTime);
}

/**
 * Applies credit updates mutation to the store.
 */
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

/**
 * Applies task property updates mutation to the store.
 */
function applyTaskUpdates(
  store: TunnelStore,
  updates: Array<{
    id: string;
    status?: string;
    credits?: number;
    desired_credits?: number;
    importance?: number;
    due_date?: string | null | undefined;
  }>,
): void {
  for (const update of updates) {
    const {id, ...props} = update;

    if (props.status === 'Done') {
      store.completeTask(id as TaskID);
      continue;
    }

    const taskProps: Partial<PersistedTask> = {};

    if (props.status) {
      taskProps.status =
        StoreTaskStatus[props.status as keyof typeof StoreTaskStatus];
    }
    if (props.credits !== undefined) {
      taskProps.credits = props.credits;
    }
    if (props.desired_credits !== undefined) {
      taskProps.desiredCredits = props.desired_credits;
    }
    if (props.importance !== undefined) {
      taskProps.importance = props.importance;
    }
    if (props.due_date !== undefined) {
      const existingTask = store.getTask(id as TaskID);
      if (existingTask) {
        taskProps.schedule = {
          ...existingTask.schedule,
          dueDate: props.due_date
            ? new Date(props.due_date).getTime()
            : undefined,
        };
      }
    }

    store.updateTask(id as TaskID, taskProps);
  }
}

/**
 * Parses view filter from test step.
 */
function parseViewFilter(filterInput?: string): ViewFilter {
  if (!filterInput) {
    return {placeId: 'All'};
  }
  return {
    placeId: filterInput === 'All Places' ? 'All' : (filterInput as PlaceID),
  };
}

/**
 * Asserts expected task properties against computed tasks.
 */
function assertExpectedProps(
  computedMap: Map<TaskID, EnrichedTask>,
  expectedProps: Array<{
    id: string;
    score?: number;
    effective_credits?: number;
    normalized_importance?: number;
    is_visible?: boolean;
    is_ready?: boolean;
  }>,
): void {
  for (const expected of expectedProps) {
    const task = computedMap.get(expected.id as TaskID);
    expect(
      task,
      `Task ${expected.id} should be in computed results`,
    ).toBeDefined();

    if (!task) continue;

    for (const key of Object.keys(expected) as Array<keyof typeof expected>) {
      if (key === 'id') continue;

      switch (key) {
        case 'score':
          expect(task.priority, `Task ${expected.id} priority`).toBeCloseTo(
            expected.score as number,
            4,
          );
          break;
        case 'effective_credits':
          expect(
            task.effectiveCredits,
            `Task ${expected.id} effectiveCredits`,
          ).toBeCloseTo(expected.effective_credits as number, 4);
          break;
        case 'normalized_importance':
          expect(
            task.normalizedImportance,
            `Task ${expected.id} normalizedImportance`,
          ).toBeCloseTo(expected.normalized_importance as number, 4);
          break;
        case 'is_visible':
          expect(task.visibility, `Task ${expected.id} is_visible`).toBe(
            expected.is_visible as boolean,
          );
          break;
        case 'is_ready':
          expect(task.isReady, `Task ${expected.id} is_ready`).toBe(
            expected.is_ready as boolean,
          );
          break;
        default:
          throw new Error(
            `Unhandled expected property '${key}' in test fixture for task '${expected.id}'`,
          );
      }
    }
  }
}

describe('Algorithm Test Suite', () => {
  for (const fixtureFile of fixtureFiles) {
    const fixtureName = basename(fixtureFile, '.yaml');
    const yamlContent = readFileSync(join(FIXTURES_PATH, fixtureFile), 'utf8');
    const testCase = load(yamlContent);

    if (!validate(testCase)) {
      throw new Error(
        `Invalid fixture ${fixtureName}: ${ajv.errorsText(validate.errors)}`,
      );
    }

    const validTestCase = testCase as TunnelAlgorithmTestCaseSchema;

    describe(`Fixture: ${fixtureName} - ${validTestCase.name}`, () => {
      let store: TunnelStore;
      let testStartDate: Date;

      beforeAll(() => {
        // Reset Time
        testStartDate = new Date(validTestCase.initial_state.current_time);
        mockCurrentTimestamp(testStartDate.getTime());

        // Initialize Store with initial state
        const initialTasks: Record<string, PersistedTask> = {};

        for (const taskInput of validTestCase.initial_state.tasks) {
          const parsedTasks = parseTaskInput(taskInput, testStartDate);
          for (const t of parsedTasks) {
            initialTasks[t.id] = t;
          }
        }

        const initialPlaces: Record<string, Place> = {};
        if (validTestCase.initial_state.places) {
          for (const p of validTestCase.initial_state.places) {
            const place = parsePlaceInput(p);
            initialPlaces[place.id] = place;
          }
        }

        const rootTaskIds = Object.values(initialTasks)
          .filter(t => !t.parentId)
          .map(t => t.id);

        store = new TunnelStore({
          tasks: initialTasks,
          places: initialPlaces,
          rootTaskIds,
          nextTaskId: 1,
          nextPlaceId: 1,
        });
      });

      afterAll(() => {
        resetCurrentTimestampMock();
      });

      for (const [stepIndex, step] of validTestCase.steps.entries()) {
        it(`Step ${String(stepIndex + 1)}: ${step.name}`, () => {
          // Apply mutations
          if (step.mutation?.advance_time_seconds) {
            applyTimeAdvancement(step.mutation.advance_time_seconds);
          }

          if (step.mutation?.update_credits) {
            applyCreditUpdates(store, step.mutation.update_credits);
          }

          if (step.mutation?.task_updates) {
            applyTaskUpdates(store, step.mutation.task_updates);
          }

          // Compute results
          const viewFilter = parseViewFilter(step.view_filter);
          // Get ALL tasks to verify properties (even on hidden/zero-priority tasks)
          const allTasks = store.dumpCalculatedStateForTest(viewFilter, {
            includeHidden: true,
            includeDone: true,
          });
          
          const computedMap = new Map<TaskID, EnrichedTask>();
          for (const t of allTasks) {
            computedMap.set(t.id, t as EnrichedTask);
          }

          if (step.expected_props) {
            // Assert expectations on any task (visible or hidden)
            assertExpectedProps(computedMap, step.expected_props);
          }

          if (step.expected_order) {
            // Filter to what the user would actually see (The "Do List")
            // Use the Store's public API which respects domain rules logic (including thresholds)
            const visibleTasks = store.getTodoListForTest(viewFilter);
            const visibleIds = visibleTasks.map(t => t.id);

            expect(visibleIds).toEqual(step.expected_order);
          }
        });
      }
    });
  }
});
