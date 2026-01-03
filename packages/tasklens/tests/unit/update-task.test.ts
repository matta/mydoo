/**
 * Exhaustive unit tests for ops.updateTask
 *
 * These tests verify:
 * 1. All Task fields are correctly updated via direct proxy mutation
 * 2. Optional field deletion semantics (using 'in' operator checks)
 * 3. Nested object handling (schedule)
 * 4. Round-trip serialization through Automerge save/load
 */
import {
  type PersistedTask,
  type PlaceID,
  type TaskID,
  TaskStatus,
} from '@mydoo/tasklens';
import {beforeEach, describe, expect, it} from 'vitest';

import {createTask, updateTask} from '../../src/persistence/ops';
import {TunnelStore} from '../../src/persistence/store';

describe('updateTask - Exhaustive Field Coverage', () => {
  let store: TunnelStore;
  let baseTask: PersistedTask;

  beforeEach(() => {
    store = new TunnelStore();
    baseTask = createTask(store.state, {
      title: 'Base Task',
      importance: 0.5,
      creditIncrement: 0.3,
      credits: 10,
      desiredCredits: 20,
      creditsTimestamp: 1000,
      priorityTimestamp: 2000,
      isSequential: false,
      isAcknowledged: false,
    });
  });

  describe('Required String Fields', () => {
    it('should update title', () => {
      updateTask(store.state, baseTask.id, {title: 'Updated Title'});
      expect(store.state.tasks[baseTask.id]?.title).toBe('Updated Title');
    });

    it('should update status to Done and set lastCompletedAt', () => {
      const before = Date.now();
      updateTask(store.state, baseTask.id, {status: TaskStatus.Done});
      const task = store.state.tasks[baseTask.id];
      expect(task?.status).toBe(TaskStatus.Done);
      expect(task?.lastCompletedAt).toBeDefined();
      expect(task?.lastCompletedAt).toBeGreaterThanOrEqual(before);
    });

    it('should update status to Pending', () => {
      // First set to Done
      updateTask(store.state, baseTask.id, {status: TaskStatus.Done});
      // Then back to Pending
      updateTask(store.state, baseTask.id, {status: TaskStatus.Pending});
      expect(store.state.tasks[baseTask.id]?.status).toBe(TaskStatus.Pending);
    });
  });

  describe('Required Numeric Fields', () => {
    it('should update importance', () => {
      updateTask(store.state, baseTask.id, {importance: 0.9});
      expect(store.state.tasks[baseTask.id]?.importance).toBe(0.9);
    });

    it('should update importance to 0', () => {
      updateTask(store.state, baseTask.id, {importance: 0});
      expect(store.state.tasks[baseTask.id]?.importance).toBe(0);
    });

    it('should update importance to 1', () => {
      updateTask(store.state, baseTask.id, {importance: 1});
      expect(store.state.tasks[baseTask.id]?.importance).toBe(1);
    });

    it('should throw for invalid importance < 0', () => {
      expect(() =>
        updateTask(store.state, baseTask.id, {importance: -0.1}),
      ).toThrow('Importance must be between 0.0 and 1.0.');
    });

    it('should throw for invalid importance > 1', () => {
      expect(() =>
        updateTask(store.state, baseTask.id, {importance: 1.1}),
      ).toThrow('Importance must be between 0.0 and 1.0.');
    });

    it('should update creditIncrement', () => {
      updateTask(store.state, baseTask.id, {creditIncrement: 0.75});
      expect(store.state.tasks[baseTask.id]?.creditIncrement).toBe(0.75);
    });

    it('should update creditIncrement to 0', () => {
      updateTask(store.state, baseTask.id, {creditIncrement: 0});
      expect(store.state.tasks[baseTask.id]?.creditIncrement).toBe(0);
    });

    it('should throw for negative creditIncrement', () => {
      expect(() =>
        updateTask(store.state, baseTask.id, {creditIncrement: -1}),
      ).toThrow('CreditIncrement cannot be negative.');
    });

    it('should update credits', () => {
      updateTask(store.state, baseTask.id, {credits: 100});
      expect(store.state.tasks[baseTask.id]?.credits).toBe(100);
    });

    it('should update desiredCredits', () => {
      updateTask(store.state, baseTask.id, {desiredCredits: 50});
      expect(store.state.tasks[baseTask.id]?.desiredCredits).toBe(50);
    });

    it('should throw for negative desiredCredits', () => {
      expect(() =>
        updateTask(store.state, baseTask.id, {desiredCredits: -10}),
      ).toThrow('DesiredCredits cannot be negative.');
    });

    it('should update creditsTimestamp', () => {
      updateTask(store.state, baseTask.id, {creditsTimestamp: 5000});
      expect(store.state.tasks[baseTask.id]?.creditsTimestamp).toBe(5000);
    });

    it('should update priorityTimestamp', () => {
      updateTask(store.state, baseTask.id, {priorityTimestamp: 6000});
      expect(store.state.tasks[baseTask.id]?.priorityTimestamp).toBe(6000);
    });
  });

  describe('Boolean Fields', () => {
    it('should update isSequential to true', () => {
      updateTask(store.state, baseTask.id, {isSequential: true});
      expect(store.state.tasks[baseTask.id]?.isSequential).toBe(true);
    });

    it('should update isSequential to false', () => {
      updateTask(store.state, baseTask.id, {isSequential: true});
      updateTask(store.state, baseTask.id, {isSequential: false});
      expect(store.state.tasks[baseTask.id]?.isSequential).toBe(false);
    });

    it('should update isAcknowledged to true', () => {
      updateTask(store.state, baseTask.id, {isAcknowledged: true});
      expect(store.state.tasks[baseTask.id]?.isAcknowledged).toBe(true);
    });

    it('should update isAcknowledged to false', () => {
      updateTask(store.state, baseTask.id, {isAcknowledged: true});
      updateTask(store.state, baseTask.id, {isAcknowledged: false});
      expect(store.state.tasks[baseTask.id]?.isAcknowledged).toBe(false);
    });
  });

  describe('Nested Object: schedule', () => {
    it('should update schedule.type', () => {
      updateTask(store.state, baseTask.id, {
        schedule: {type: 'Routinely', leadTime: 1000},
      });
      expect(store.state.tasks[baseTask.id]?.schedule.type).toBe('Routinely');
    });

    it('should update schedule.leadTime', () => {
      updateTask(store.state, baseTask.id, {
        schedule: {type: 'Once', leadTime: 86400000},
      });
      expect(store.state.tasks[baseTask.id]?.schedule.leadTime).toBe(86400000);
    });

    it('should set schedule.dueDate', () => {
      const dueDate = Date.now() + 86400000;
      updateTask(store.state, baseTask.id, {
        schedule: {type: 'Once', leadTime: 1000, dueDate},
      });
      expect(store.state.tasks[baseTask.id]?.schedule.dueDate).toBe(dueDate);
    });

    it('should delete schedule.dueDate when explicitly set to undefined', () => {
      // First set a dueDate
      const dueDate = Date.now();
      updateTask(store.state, baseTask.id, {
        schedule: {type: 'Once', leadTime: 1000, dueDate},
      });
      expect(store.state.tasks[baseTask.id]?.schedule.dueDate).toBe(dueDate);

      // Then explicitly unset it
      updateTask(store.state, baseTask.id, {
        schedule: {type: 'Once', leadTime: 1000, dueDate: undefined},
      });
      expect(store.state.tasks[baseTask.id]?.schedule.dueDate).toBeUndefined();
      // Verify key is actually deleted (important for Automerge)
      expect(
        'dueDate' in (store.state.tasks[baseTask.id]?.schedule ?? {}),
      ).toBe(false);
    });

    it('should NOT touch dueDate if not passed in schedule', () => {
      // First set a dueDate
      const dueDate = Date.now();
      updateTask(store.state, baseTask.id, {
        schedule: {type: 'Once', leadTime: 1000, dueDate},
      });
      expect(store.state.tasks[baseTask.id]?.schedule.dueDate).toBe(dueDate);

      // Update only leadTime - dueDate should remain
      updateTask(store.state, baseTask.id, {
        schedule: {type: 'Once', leadTime: 2000},
      });
      expect(store.state.tasks[baseTask.id]?.schedule.dueDate).toBe(dueDate);
    });
  });

  describe('Initialization Logic', () => {
    it('should default dueDate to Now when switching to "Routinely"', () => {
      updateTask(store.state, baseTask.id, {
        schedule: {type: 'Routinely', leadTime: 1000},
      });
      const task = store.state.tasks[baseTask.id];
      expect(task?.schedule.dueDate).toBeDefined();
      expect(task?.schedule.dueDate).toBeLessThanOrEqual(Date.now());
    });

    it('should NOT overwrite existing dueDate when switching to "Routinely"', () => {
      const future = Date.now() + 10000;
      updateTask(store.state, baseTask.id, {
        schedule: {type: 'Routinely', leadTime: 1000, dueDate: future},
      });
      expect(store.state.tasks[baseTask.id]?.schedule.dueDate).toBe(future);
    });
  });

  describe('Optional Fields: parentId', () => {
    it('should not modify parentId if not passed', () => {
      const child = createTask(store.state, {
        title: 'Child',
        parentId: baseTask.id,
      });
      updateTask(store.state, child.id, {title: 'Updated Child'});
      expect(store.state.tasks[child.id]?.parentId).toBe(baseTask.id);
    });

    it('should delete parentId when explicitly set to undefined via "in" check', () => {
      const child = createTask(store.state, {
        title: 'Child',
        parentId: baseTask.id,
      });
      expect(store.state.tasks[child.id]?.parentId).toBe(baseTask.id);

      // Explicitly unset parentId by passing it as undefined with 'in' semantics
      const propsWithExplicitUndefined: Partial<PersistedTask> = {};
      Object.defineProperty(propsWithExplicitUndefined, 'parentId', {
        value: undefined,
        enumerable: true,
      });
      updateTask(store.state, child.id, propsWithExplicitUndefined);

      expect(store.state.tasks[child.id]?.parentId).toBeUndefined();
      expect('parentId' in (store.state.tasks[child.id] ?? {})).toBe(false);
    });
  });

  describe('Optional Fields: placeId', () => {
    it('should update placeId', () => {
      const newPlaceId = 'Office' as PlaceID;
      updateTask(store.state, baseTask.id, {placeId: newPlaceId});
      expect(store.state.tasks[baseTask.id]?.placeId).toBe(newPlaceId);
    });

    it('should not modify placeId if not passed', () => {
      const originalPlaceId = store.state.tasks[baseTask.id]?.placeId;
      updateTask(store.state, baseTask.id, {title: 'New Title'});
      expect(store.state.tasks[baseTask.id]?.placeId).toBe(originalPlaceId);
    });

    it('should delete placeId when explicitly set to undefined via "in" check', () => {
      // First ensure placeId is set
      const newPlaceId = 'Office' as PlaceID;
      updateTask(store.state, baseTask.id, {placeId: newPlaceId});
      expect(store.state.tasks[baseTask.id]?.placeId).toBe(newPlaceId);

      // Explicitly unset placeId
      const propsWithExplicitUndefined: Partial<PersistedTask> = {};
      Object.defineProperty(propsWithExplicitUndefined, 'placeId', {
        value: undefined,
        enumerable: true,
      });
      updateTask(store.state, baseTask.id, propsWithExplicitUndefined);

      expect(store.state.tasks[baseTask.id]?.placeId).toBeUndefined();
      expect('placeId' in (store.state.tasks[baseTask.id] ?? {})).toBe(false);
    });
  });

  describe('Error Handling', () => {
    it('should throw for non-existent task', () => {
      expect(() =>
        updateTask(store.state, 'non-existent' as TaskID, {title: 'New'}),
      ).toThrow('Task with ID non-existent not found.');
    });
  });

  describe('Multiple Fields Update', () => {
    it('should update multiple fields atomically', () => {
      updateTask(store.state, baseTask.id, {
        title: 'Multi Update',
        importance: 0.8,
        status: TaskStatus.Done,
        isSequential: true,
      });

      const task = store.state.tasks[baseTask.id];
      expect(task?.title).toBe('Multi Update');
      expect(task?.importance).toBe(0.8);
      expect(task?.status).toBe(TaskStatus.Done);
      expect(task?.isSequential).toBe(true);
    });
  });
});

describe('updateTask - Round-Trip Serialization (Automerge)', () => {
  /**
   * Round-trip helper using actual Automerge serialization.
   * This saves the store to binary format and loads it back.
   */
  function automergeRoundTrip(store: TunnelStore): TunnelStore {
    const bytes = store.save();
    return TunnelStore.load(bytes);
  }

  let store: TunnelStore;
  let baseTask: PersistedTask;

  beforeEach(() => {
    store = new TunnelStore();
    // Use store.createTask to properly wrap in Automerge.change
    baseTask = store.createTask({
      title: 'Serialization Test',
      importance: 0.7,
      creditIncrement: 0.4,
    });
  });

  it('should survive Automerge round-trip for all required fields', () => {
    // Update all required fields using TunnelStore wrapper
    store.updateTask(baseTask.id, {
      title: 'Round-Trip Title',
      importance: 0.95,
      creditIncrement: 0.6,
      credits: 25,
      desiredCredits: 50,
      creditsTimestamp: 123456789,
      priorityTimestamp: 987654321,
      status: TaskStatus.Done,
      isSequential: true,
      isAcknowledged: true,
      schedule: {
        type: 'Routinely',
        leadTime: 604800000, // 7 days
        dueDate: Date.now() + 86400000,
      },
    });

    const taskBefore = store.state.tasks[baseTask.id];
    if (!taskBefore) throw new Error('Task should exist before save');

    // Actual Automerge round-trip
    const restoredStore = automergeRoundTrip(store);
    const taskAfter = restoredStore.state.tasks[baseTask.id];
    if (!taskAfter) throw new Error('Task should exist after load');

    // Verify all fields survive Automerge serialization
    expect(taskAfter.title).toBe(taskBefore.title);
    expect(taskAfter.importance).toBe(taskBefore.importance);
    expect(taskAfter.creditIncrement).toBe(taskBefore.creditIncrement);
    expect(taskAfter.credits).toBe(taskBefore.credits);
    expect(taskAfter.desiredCredits).toBe(taskBefore.desiredCredits);
    expect(taskAfter.creditsTimestamp).toBe(taskBefore.creditsTimestamp);
    expect(taskAfter.priorityTimestamp).toBe(taskBefore.priorityTimestamp);
    expect(taskAfter.status).toBe(taskBefore.status);
    expect(taskAfter.isSequential).toBe(taskBefore.isSequential);
    expect(taskAfter.isAcknowledged).toBe(taskBefore.isAcknowledged);
    expect(taskAfter.schedule.type).toBe(taskBefore.schedule.type);
    expect(taskAfter.schedule.leadTime).toBe(taskBefore.schedule.leadTime);
    expect(taskAfter.schedule.dueDate).toBe(taskBefore.schedule.dueDate);
  });

  it('should survive Automerge round-trip with optional fields present', () => {
    const placeId = 'TestPlace' as PlaceID;
    const child = store.createTask({
      title: 'Child Task',
      parentId: baseTask.id,
      placeId,
    });

    const taskBefore = store.state.tasks[child.id];
    if (!taskBefore) throw new Error('Task should exist before save');

    const restoredStore = automergeRoundTrip(store);
    const taskAfter = restoredStore.state.tasks[child.id];
    if (!taskAfter) throw new Error('Task should exist after load');

    expect(taskAfter.parentId).toBe(taskBefore.parentId);
    expect(taskAfter.placeId).toBe(taskBefore.placeId);
  });

  it('should survive Automerge round-trip with optional fields absent', () => {
    // Create a root task (no parentId)
    const rootTask = store.createTask({
      title: 'Root Only',
    });

    const taskBefore = store.state.tasks[rootTask.id];
    if (!taskBefore) throw new Error('Task should exist before save');

    const restoredStore = automergeRoundTrip(store);
    const taskAfter = restoredStore.state.tasks[rootTask.id];
    if (!taskAfter) throw new Error('Task should exist after load');

    // After Automerge round-trip, absent keys should still be absent
    expect(taskAfter.parentId).toBeUndefined();
    // Verify the key is actually not present (Automerge respects delete)
    expect('parentId' in taskAfter).toBe(false);
  });

  it('should reflect mutations correctly after Automerge round-trip', () => {
    // Update task
    store.updateTask(baseTask.id, {title: 'Before Save'});

    // Automerge save/load cycle
    const restoredStore = automergeRoundTrip(store);

    // Verify state was preserved
    const restoredTask = restoredStore.state.tasks[baseTask.id];
    expect(restoredTask?.title).toBe('Before Save');
  });

  it('should correctly handle all Task fields through Automerge', () => {
    // Create a task and set ALL possible fields
    const fullTask = store.createTask({title: 'Reflection Test'});
    const fullId = fullTask.id;

    // Update with every possible field
    store.updateTask(fullId, {
      title: 'Full Reflection',
      status: TaskStatus.Done,
      importance: 0.99,
      creditIncrement: 0.88,
      credits: 777,
      desiredCredits: 999,
      creditsTimestamp: 111111,
      priorityTimestamp: 222222,
      isSequential: true,
      isAcknowledged: true,
      placeId: 'ReflectionPlace' as PlaceID,
      schedule: {
        type: 'Routinely',
        leadTime: 1000000,
        dueDate: 2000000,
      },
    });

    // Automerge round-trip
    const restoredStore = automergeRoundTrip(store);
    const restored = restoredStore.state.tasks[fullId];
    if (!restored) throw new Error('Task should exist after load');

    // Use Object.keys to verify all expected fields are present
    const expectedKeys = [
      'id',
      'title',
      'status',
      'importance',
      'creditIncrement',
      'credits',
      'desiredCredits',
      'creditsTimestamp',
      'priorityTimestamp',
      'isSequential',
      'isAcknowledged',
      'placeId',
      'schedule',
      'childTaskIds',
      'lastCompletedAt',
    ];

    for (const key of expectedKeys) {
      expect(key in restored).toBe(true);
    }

    // Verify schedule nested keys
    expect('type' in restored.schedule).toBe(true);
    expect('leadTime' in restored.schedule).toBe(true);
    expect('dueDate' in restored.schedule).toBe(true);
  });
});

describe('updateTask - Automerge Deletion Semantics', () => {
  let store: TunnelStore;

  beforeEach(() => {
    store = new TunnelStore();
  });

  it('should use delete operator for optional fields (verified by key absence)', () => {
    const task = createTask(store.state, {
      title: 'Delete Test',
      placeId: 'ToBeDeleted' as PlaceID,
    });

    // Verify placeId exists
    const storedTask = store.state.tasks[task.id];
    if (!storedTask) throw new Error('Task should exist');
    expect('placeId' in storedTask).toBe(true);

    // Explicitly delete via update
    const props: Partial<PersistedTask> = {};
    Object.defineProperty(props, 'placeId', {
      value: undefined,
      enumerable: true,
    });
    updateTask(store.state, task.id, props);

    // Verify key is actually deleted (not just undefined)
    const updatedTask = store.state.tasks[task.id];
    if (!updatedTask) throw new Error('Task should exist');
    expect('placeId' in updatedTask).toBe(false);
  });

  it('should preserve other fields when deleting optional field', () => {
    const task = createTask(store.state, {
      title: 'Preserve Test',
      importance: 0.9,
      placeId: 'ToBeDeleted' as PlaceID,
    });

    const props: Partial<PersistedTask> = {};
    Object.defineProperty(props, 'placeId', {
      value: undefined,
      enumerable: true,
    });
    updateTask(store.state, task.id, props);

    // Other fields should be preserved
    expect(store.state.tasks[task.id]?.title).toBe('Preserve Test');
    expect(store.state.tasks[task.id]?.importance).toBe(0.9);
  });
});
