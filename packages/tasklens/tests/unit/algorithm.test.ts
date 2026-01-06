import {readdirSync, readFileSync} from 'node:fs';
import {basename, join} from 'node:path';
import Ajv from 'ajv';
import addFormats from 'ajv-formats';
import {dump, load} from 'js-yaml';
import {afterAll, beforeAll, describe, expect, it} from 'vitest';
import schemaJson from '../../specs/compliance/schemas/test-case.schema.json';
import {getPrioritizedTasks} from '../../src/domain/priority';
import type {TunnelAlgorithmFeatureSchema} from '../../src/generated/feature';
import type {
  Place as PlaceInput,
  TaskInput,
  TunnelAlgorithmTestCaseSchema,
} from '../../src/generated/test-case';
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
import {convertFeatureToLegacy} from './converters';

const ajv = new Ajv();
addFormats(ajv);
const validate = ajv.compile(schemaJson);

const FIXTURES_PATH = join(process.cwd(), 'specs', 'compliance', 'fixtures');

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
 * Type for task updates that may come from both legacy and feature fixtures.
 * Includes the standard schema properties plus additional properties from
 * feature fixtures like lead_time_seconds and place_id.
 */
interface TaskUpdate {
  id: string;
  status?: string;
  credits?: number | string;
  desired_credits?: number | string;
  importance?: number | string;
  due_date?: string | null;
  is_acknowledged?: boolean | string;
  lead_time_seconds?: number | string;
  place_id?: string | null;
}

/**
 * Applies task property updates mutation to the store.
 */
function applyTaskUpdates(store: TunnelStore, updates: TaskUpdate[]): void {
  for (const update of updates) {
    const {id, ...props} = update;

    const taskProps: Partial<PersistedTask> = {};

    if (props.status) {
      taskProps.status =
        StoreTaskStatus[props.status as keyof typeof StoreTaskStatus];
    }
    if (props.credits !== undefined) {
      taskProps.credits = Number(props.credits);
    }
    if (props.desired_credits !== undefined) {
      taskProps.desiredCredits = Number(props.desired_credits);
    }
    if (props.importance !== undefined) {
      taskProps.importance = Number(props.importance);
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
    if (props.lead_time_seconds !== undefined) {
      const existingTask = store.getTask(id as TaskID);
      if (existingTask) {
        taskProps.schedule = {
          ...existingTask.schedule,
          ...(taskProps.schedule || {}),
          leadTime: Number(props.lead_time_seconds) * 1000,
        };
      }
    }
    if (props.place_id !== undefined) {
      taskProps.placeId = props.place_id as PlaceID;
    }

    if (props.is_acknowledged !== undefined) {
      taskProps.isAcknowledged = Boolean(props.is_acknowledged);
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
  const allFiles = readdirSync(FIXTURES_PATH).filter(f => f.endsWith('.yaml'));

  for (const fixtureFile of allFiles) {
    const fixtureName = basename(fixtureFile, '.yaml');
    const yamlContent = readFileSync(join(FIXTURES_PATH, fixtureFile), 'utf8');
    const isFeature = fixtureFile.endsWith('.feature.yaml');
    let testCases: TunnelAlgorithmTestCaseSchema[] = [];

    if (isFeature) {
      const feature = load(yamlContent) as TunnelAlgorithmFeatureSchema;
      // Convert to legacy structure (array of scenarios -> array of test cases)
      testCases = convertFeatureToLegacy(feature);

      // Round Trip Check: Ensure converted legacy cases are valid YAML
      testCases = testCases.map(tc => {
        const str = dump(tc);
        return load(str) as TunnelAlgorithmTestCaseSchema;
      });
    } else {
      const legacy = load(yamlContent) as TunnelAlgorithmTestCaseSchema;

      if (!validate(legacy)) {
        throw new Error(
          `Invalid fixture ${fixtureName}: ${ajv.errorsText(validate.errors)}`,
        );
      }

      // Round Trip Check
      const str = dump(legacy);
      testCases = [load(str) as TunnelAlgorithmTestCaseSchema];
    }

    for (const validTestCase of testCases) {
      describe(`Fixture: ${fixtureName} - ${validTestCase.name}`, () => {
        let store: TunnelStore;
        let testStartDate: Date;

        beforeAll(() => {
          // Reset Time
          testStartDate = new Date(
            validTestCase.initial_state.current_time || '2025-01-01T12:00:00Z',
          );
          mockCurrentTimestamp(testStartDate.getTime());

          // Initialize Store with initial state
          const initialTasks: Record<string, PersistedTask> = {};

          if (validTestCase.initial_state.tasks) {
            for (const taskInput of validTestCase.initial_state.tasks) {
              const parsedTasks = parseTaskInput(taskInput, testStartDate);
              for (const t of parsedTasks) {
                initialTasks[t.id] = t;
              }
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
              applyTimeAdvancement(Number(step.mutation.advance_time_seconds));
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
            const allTasks = getPrioritizedTasks(store.doc, viewFilter, {
              includeHidden: true,
              mode: 'plan-outline',
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
              const visibleTasks = getPrioritizedTasks(store.doc, viewFilter);
              const visibleIds = visibleTasks.map(t => t.id);

              expect(visibleIds).toEqual(step.expected_order);
            }
          });
        }
      });
    }
  }
});
