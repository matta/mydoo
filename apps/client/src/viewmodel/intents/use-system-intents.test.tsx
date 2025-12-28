import {type DocHandle, Repo} from '@automerge/automerge-repo';
import {
  createStore,
  type DocumentHandle,
  type TaskID,
  TaskStatus,
  type TunnelState,
} from '@mydoo/tasklens';
import {act, renderHook, waitFor} from '@testing-library/react';
import {afterEach, beforeEach, describe, expect, it} from 'vitest';

import {createTestWrapper} from '../../test/setup';
import {useSystemIntents} from './use-system-intents';

const createMockTask = (
  id: string,
  title: string,
  status: TaskStatus,
  isAcknowledged: boolean,
): any => ({
  id: id as TaskID,
  title,
  status,
  isAcknowledged,
  childTaskIds: [],
  schedule: {type: 'Once', leadTime: 0},
  importance: 0.5,
  credits: 0,
  desiredCredits: 0,
  creditIncrement: 1,
  creditsTimestamp: 0,
  priorityTimestamp: 0,
  isSequential: false,
  notes: '',
});

describe('useSystemIntents', () => {
  let repo: Repo;
  let handle: DocHandle<TunnelState>;
  let docId: DocumentHandle;

  beforeEach(() => {
    repo = new Repo({network: []});
    window.location.hash = '';
    handle = repo.create({tasks: {}, rootTaskIds: [], places: {}});
    docId = handle.url as unknown as DocumentHandle;
  });

  afterEach(() => {
    window.location.hash = '';
  });

  describe('refreshTaskList', () => {
    it('should acknowledge completed tasks', async () => {
      // 1. Seed Data
      handle.change(d => {
        d.tasks['task1' as TaskID] = createMockTask(
          'task1',
          'Pending',
          TaskStatus.Pending,
          false,
        );
        d.tasks['task2' as TaskID] = createMockTask(
          'task2',
          'Done Unacked',
          TaskStatus.Done,
          false,
        );
        d.tasks['task3' as TaskID] = createMockTask(
          'task3',
          'Done Acked',
          TaskStatus.Done,
          true,
        );
        d.rootTaskIds = [
          'task1' as TaskID,
          'task2' as TaskID,
          'task3' as TaskID,
        ];
      });

      // 2. Setup Hook
      const store = createStore();
      const wrapper = createTestWrapper(repo, store, docId);
      const {result} = renderHook(() => useSystemIntents(), {wrapper});

      // Wait for Redux to have the tasks (to avoid race conditions in intents)
      await waitFor(() => {
        const state = store.getState();
        if (!state.tasks.entities['task1' as TaskID])
          throw new Error('Task1 not in store');
      });

      // 3. Act
      act(() => {
        result.current.refreshTaskList();
      });

      // 4. Verify in Doc
      await waitFor(() => {
        const doc = handle.doc();
        const t1 = doc.tasks['task1' as TaskID];
        const t2 = doc.tasks['task2' as TaskID];
        const t3 = doc.tasks['task3' as TaskID];

        if (!t1 || !t2 || !t3) throw new Error('Tasks missing in final doc');
        expect(t1.isAcknowledged).toBe(false);
        expect(t2.isAcknowledged).toBe(true); // Changed!
        expect(t3.isAcknowledged).toBe(true);
      });
    });
  });
});
