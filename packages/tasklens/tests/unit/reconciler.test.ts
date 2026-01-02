import type {DocHandle} from '@automerge/automerge-repo';
import {describe, expect, it, vi} from 'vitest';
import {runReconciler} from '../../src/domain/reconciler';
import type {TunnelState} from '../../src/types';

// Mock DocHandle
class MockDocHandle {
  // biome-ignore lint/suspicious/noExplicitAny: Mocking internal state
  private _doc: any;

  // biome-ignore lint/suspicious/noExplicitAny: Mocking internal state
  constructor(initialDoc: any) {
    this._doc = initialDoc;
  }

  doc() {
    return this._doc;
  }

  // biome-ignore lint/suspicious/noExplicitAny: Mocking internal state
  change(callback: (doc: any) => void) {
    // Deep clone to simulate immutable state transition base
    // In real Automerge, this is handled by the library.
    // For tests, we just mutate _doc in place as checking the result is what matters.
    callback(this._doc);
  }
}

describe('runReconciler', () => {
  it('should migrate Recurring tasks to Routinely', () => {
    const initialDoc = {
      tasks: {
        'task-1': {
          id: 'task-1',
          title: 'Old Task',
          status: 'Pending',
          schedule: {
            type: 'Recurring', // Old type
            leadTime: 1000,
          },
        },
        'task-2': {
          id: 'task-2',
          title: 'Normal Task',
          status: 'Pending',
          schedule: {
            type: 'Once',
            leadTime: 0,
          },
        },
      },
    };

    // Use 'as any' to bypass double-cast lint rule and satisfy type system
    // biome-ignore lint/suspicious/noExplicitAny: Mock needs force cast
    const handle: DocHandle<TunnelState> = new MockDocHandle(initialDoc) as any;
    const result = runReconciler(handle);

    expect(result).toBe(true); // Should return true on mutation

    // biome-ignore lint/suspicious/noExplicitAny: Accessing migrated property
    const updatedTask = initialDoc.tasks['task-1'] as any;
    expect(updatedTask.schedule.type).toBe('Routinely');

    const normalTask = initialDoc.tasks['task-2'];
    expect(normalTask.schedule.type).toBe('Once');
  });

  it('should backfill lastCompletedAt for Done Routinely tasks', () => {
    const initialDoc = {
      tasks: {
        'task-done': {
          id: 'task-done',
          title: 'Done Old Task',
          status: 'Done',
          // Missing lastCompletedAt
          schedule: {
            type: 'Recurring',
            leadTime: 1000,
          },
        },
      },
    };

    // Use 'as any' to bypass double-cast lint rule and satisfy type system
    // biome-ignore lint/suspicious/noExplicitAny: Mock needs force cast
    const handle: DocHandle<TunnelState> = new MockDocHandle(initialDoc) as any;
    const result = runReconciler(handle);

    expect(result).toBe(true); // Should return true on mutation

    // biome-ignore lint/suspicious/noExplicitAny: Accessing migrated property
    const task = initialDoc.tasks['task-done'] as any;
    expect(task.schedule.type).toBe('Routinely');
    expect(task.lastCompletedAt).toBeDefined();
    // Should be close to now
    expect(task.lastCompletedAt).toBeGreaterThan(Date.now() - 1000);
  });

  it('should be idempotent (not change anything if already migrated)', () => {
    const initialDoc = {
      tasks: {
        'task-1': {
          id: 'task-1',
          title: 'New Task',
          status: 'Pending',
          schedule: {
            type: 'Routinely',
            leadTime: 1000,
          },
        },
      },
    };

    // Use 'as any' to bypass double-cast lint rule and satisfy type system
    // biome-ignore lint/suspicious/noExplicitAny: Mock needs force cast
    const handle: DocHandle<TunnelState> = new MockDocHandle(initialDoc) as any;
    // Spy on change to ensure it's not called
    const changeSpy = vi.spyOn(handle, 'change');

    const result = runReconciler(handle);

    expect(result).toBe(false); // Should return false when no mutation
    expect(changeSpy).not.toHaveBeenCalled();
  });
});
