import {describe, expect, it} from 'vitest';
import {RepeatConfigSchema, TaskSchema} from '../../src/persistence/schemas';

describe('RepeatConfigSchema', () => {
  it('should validate daily frequency with interval', () => {
    const valid = {frequency: 'daily', interval: 1};
    const result = RepeatConfigSchema.safeParse(valid);
    expect(result.success).toBe(true);
  });

  it('should validate yearly frequency', () => {
    const valid = {frequency: 'yearly', interval: 5};
    const result = RepeatConfigSchema.safeParse(valid);
    expect(result.success).toBe(true);
  });

  it('should reject invalid frequency', () => {
    const invalid = {frequency: 'hourly', interval: 1};
    const result = RepeatConfigSchema.safeParse(invalid);
    expect(result.success).toBe(false);
  });

  it('should reject non-positive interval', () => {
    const invalid = {frequency: 'weekly', interval: 0};
    const result = RepeatConfigSchema.safeParse(invalid);
    expect(result.success).toBe(false);
  });
});

describe('TaskSchema extensions', () => {
  it('should allow optional repeatConfig', () => {
    const taskData = {
      id: 'task-1',
      title: 'Repeatable',
      notes: 'some notes',
      childTaskIds: [],
      status: 'Pending',
      importance: 0.5,
      creditIncrement: 1,
      credits: 0,
      desiredCredits: 0,
      creditsTimestamp: Date.now(),
      priorityTimestamp: Date.now(),
      schedule: {type: 'Routinely', leadTime: 1000},
      repeatConfig: {frequency: 'daily', interval: 1},
      isSequential: false,
      isAcknowledged: false,
    };
    const result = TaskSchema.safeParse(taskData);
    expect(result.success).toBe(true);
  });

  it('should validate a recurring task', () => {
    const task = {
      id: 'task-recur',
      title: 'Recur',
      notes: '',
      childTaskIds: [],
      status: 'Pending',
      importance: 0.5,
      creditIncrement: 1,
      credits: 0,
      desiredCredits: 0,
      creditsTimestamp: 0,
      priorityTimestamp: 0,
      isSequential: false,
      isAcknowledged: false,
      schedule: {
        type: 'Routinely',
        leadTime: 86400000,
      },
      repeatConfig: {
        frequency: 'daily',
        interval: 1,
      },
    };
    const result = TaskSchema.safeParse(task);
    expect(result.success).toBe(true);
  });

  it('should default notes to empty string if missing', () => {
    const taskData = {
      id: 'task-2',
      title: 'No notes',
      childTaskIds: [],
      status: 'Pending',
      importance: 0.5,
      creditIncrement: 1,
      credits: 0,
      desiredCredits: 0,
      creditsTimestamp: Date.now(),
      priorityTimestamp: Date.now(),
      schedule: {type: 'Once', leadTime: 1000},
      isSequential: false,
    };
    const result = TaskSchema.parse(taskData);
    expect(result.notes).toBe('');
  });
});
