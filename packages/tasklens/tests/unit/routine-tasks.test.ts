import {
  type Chunk,
  Repo,
  type StorageAdapterInterface,
  type StorageKey,
} from '@automerge/automerge-repo';
import {describe, expect, it} from 'vitest';

class DummyStorageAdapter implements StorageAdapterInterface {
  async load(_key: StorageKey): Promise<Uint8Array | undefined> {
    return undefined;
  }
  async save(_key: StorageKey, _data: Uint8Array) {}
  async remove(_key: StorageKey) {}
  async loadRange(_keyPrefix: StorageKey): Promise<Chunk[]> {
    return [];
  }
  async removeRange(_keyPrefix: StorageKey) {}
}

import {wakeUpRoutineTasks} from '../../src/domain/routine-tasks';
import {
  type PersistedTask,
  type TaskID,
  TaskStatus,
  type TunnelState,
} from '../../src/types';

describe('wakeUpRoutineTasks', () => {
  it('should wake up a task when it is time', () => {
    const repo = new Repo({network: [], storage: new DummyStorageAdapter()});
    const handle = repo.create<TunnelState>();

    handle.change(doc => {
      doc.tasks = {
        ['task-1' as TaskID]: {
          id: 'task-1' as TaskID,
          title: 'Test Task',
          status: TaskStatus.Done,
          isAcknowledged: true,
          schedule: {
            type: 'Routinely',
            leadTime: 1000 * 60 * 60, // 1 hour
            dueDate: 1000,
          },
          repeatConfig: {
            frequency: 'daily',
            interval: 1,
          },
          // Add required PersistedTask properties with dummy values
          childTaskIds: [],
          creditIncrement: 0,
          credits: 0,
          creditsTimestamp: 0,
          desiredCredits: 0,
          importance: 0,
          isSequential: false,

          priorityTimestamp: 0,
          notes: '',
          lastCompletedAt: Date.now() - 1000 * 60 * 60 * 24, // Completed 24 hours ago
        } as PersistedTask,
      };
    });

    wakeUpRoutineTasks(handle);

    const doc = handle.docSync();
    const task = doc?.tasks['task-1' as TaskID];

    // Should be pending now
    expect(task?.status).toBe(TaskStatus.Pending);
    expect(task?.isAcknowledged).toBe(false);

    // Due date should be updated (lastCompletedAt + 1 day)
    // 24 hours ago + 24 hours = Now.
    // Wait, let's make it clearer.
  });

  it('should NOT wake up a task if it is too early', () => {
    const repo = new Repo({network: [], storage: new DummyStorageAdapter()});
    const handle = repo.create<TunnelState>();

    handle.change(doc => {
      doc.tasks = {
        ['task-1' as TaskID]: {
          id: 'task-1' as TaskID,
          title: 'Test Task',
          status: TaskStatus.Done,
          isAcknowledged: true,
          schedule: {
            type: 'Routinely',
            leadTime: 0,
          },
          repeatConfig: {
            frequency: 'daily',
            interval: 1, // 1 day interval
          },
          childTaskIds: [],
          creditIncrement: 0,
          credits: 0,
          creditsTimestamp: 0,
          desiredCredits: 0,
          importance: 0,
          isSequential: false,
          priorityTimestamp: 0,
          notes: '',
          // Completed just now
          lastCompletedAt: Date.now(),
        } as PersistedTask,
      };
    });

    wakeUpRoutineTasks(handle);

    const doc = handle.docSync();
    const task = doc?.tasks['task-1' as TaskID];

    // Should still be done
    expect(task?.status).toBe(TaskStatus.Done);
    expect(task?.isAcknowledged).toBe(true);
  });
});
