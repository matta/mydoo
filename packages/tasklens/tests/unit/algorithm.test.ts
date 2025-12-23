import * as fs from 'node:fs';
import * as path from 'node:path';
import Ajv from 'ajv';
import addFormats from 'ajv-formats';
import * as yaml from 'js-yaml';
import {afterAll, beforeAll, describe, expect, it} from 'vitest';

import type {
  Place as PlaceInput,
  TaskInput,
  TunnelAlgorithmTestCaseSchema,
} from '../../specs/compliance/schemas/test_case';
import schemaJson from '../../specs/compliance/schemas/test_case.schema.json';
import {TunnelStore} from '../../src/persistence/store';
import {
  type Place,
  type PlaceID,
  TaskStatus as StoreTaskStatus,
  type Task,
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

const FIXTURES_PATH = path.join(
  process.cwd(),
  'specs',
  'compliance',
  'fixtures',
);
const fixtureFiles = fs
  .readdirSync(FIXTURES_PATH)
  .filter((f: string) => f.endsWith('.yaml'));

function parsePlaceInput(input: PlaceInput): Place {
  return {
    id: input.id as PlaceID,
    hours: JSON.stringify(input.hours),
    includedPlaces: (input.included_places ?? []) as PlaceID[],
  };
}

// Helper to recursively parse TaskInput to Task
function parseTaskInput(
  input: TaskInput,
  testStartDate: Date,
  parentId?: string,
): Task[] {
  const tasks: Task[] = [];

  const statusMap: Record<string, StoreTaskStatus> = {
    Pending: StoreTaskStatus.Pending,
    Done: StoreTaskStatus.Done,
    Deleted: StoreTaskStatus.Deleted,
  };

  const task: Task = {
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
    input.children.forEach(child => {
      tasks.push(...parseTaskInput(child, testStartDate, task.id));
    });
  }

  return tasks;
}

describe('Algorithm Test Suite', () => {
  fixtureFiles.forEach((fixtureFile: string) => {
    const fixtureName = path.basename(fixtureFile, '.yaml');
    const yamlContent = fs.readFileSync(
      path.join(FIXTURES_PATH, fixtureFile),
      'utf8',
    );
    const testCase = yaml.load(yamlContent);

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
        const initialTasks: Record<string, Task> = {};

        validTestCase.initial_state.tasks.forEach(taskInput => {
          const parsedTasks = parseTaskInput(taskInput, testStartDate);
          parsedTasks.forEach(t => {
            initialTasks[t.id] = t;
          });
        });

        const initialPlaces: Record<string, Place> = {};
        if (validTestCase.initial_state.places) {
          validTestCase.initial_state.places.forEach(p => {
            const place = parsePlaceInput(p);
            initialPlaces[place.id] = place;
          });
        }

        const rootTaskIds = Object.values(initialTasks)
          .filter(t => !t.parentId)
          .map(t => t.id);

        store = new TunnelStore({
          tasks: initialTasks,
          places: initialPlaces,
          rootTaskIds,
          nextTaskId: 1, // Irrelevant with string IDs
          nextPlaceId: 1,
        });
      });

      afterAll(() => {
        resetCurrentTimestampMock();
      });

      validTestCase.steps.forEach((step, stepIndex) => {
        it(`Step ${String(stepIndex + 1)}: ${step.name}`, () => {
          // 1. Advance Time
          if (step.mutation?.advance_time_seconds) {
            const newTime =
              getCurrentTimestamp() + step.mutation.advance_time_seconds * 1000;
            mockCurrentTimestamp(newTime);
          }

          // 2. Update Credits (Manual Override)
          if (step.mutation?.update_credits) {
            for (const [taskId, credits] of Object.entries(
              step.mutation.update_credits,
            )) {
              store.updateTask(taskId as TaskID, {
                credits,
                creditsTimestamp: getCurrentTimestamp(),
              });
            }
          }

          // 3. Task Updates (Status, etc.)
          if (step.mutation?.task_updates) {
            // biome-ignore lint/complexity/noExcessiveCognitiveComplexity: test helper
            step.mutation.task_updates.forEach(update => {
              const {id, ...props} = update;
              if (props.status === 'Done') {
                // 'Done' status update handled via completeTask
                store.completeTask(id as TaskID);
              } else {
                const taskProps: Partial<Task> = {};
                if (props.status)
                  taskProps.status =
                    StoreTaskStatus[
                      props.status as keyof typeof StoreTaskStatus
                    ];
                if (props.credits !== undefined)
                  taskProps.credits = props.credits;
                if (props.desired_credits !== undefined)
                  taskProps.desiredCredits = props.desired_credits;
                if (props.importance !== undefined)
                  taskProps.importance = props.importance;
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
            });
          }

          // 4. Recalculate Scores
          let viewFilter: ViewFilter = {placeId: 'All'};
          if (step.view_filter) {
            if (step.view_filter === 'All Places') {
              viewFilter = {placeId: 'All'};
            } else {
              viewFilter = {placeId: step.view_filter as PlaceID};
            }
          }
          store.recalculateScores(viewFilter);

          // 5. Assertions
          if (step.expected_props) {
            step.expected_props.forEach(expected => {
              const task = store.getTask(expected.id as TaskID);
              expect(task, `Task ${expected.id} should exist`).toBeDefined();

              if (task) {
                if (expected.score !== undefined) {
                  expect(
                    task.priority,
                    `Task ${expected.id} priority`,
                  ).toBeCloseTo(expected.score, 4);
                }
                if (expected.effective_credits !== undefined) {
                  expect(
                    task.effectiveCredits,
                    `Task ${expected.id} effectiveCredits`,
                  ).toBeCloseTo(expected.effective_credits, 4);
                }
                if (expected.normalized_importance !== undefined) {
                  expect(
                    task.normalizedImportance,
                    `Task ${expected.id} normalizedImportance`,
                  ).toBeCloseTo(expected.normalized_importance, 4);
                }
              }
            });
          }
        });
      });
    });
  });
});
