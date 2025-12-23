import type {DocumentId} from '@automerge/automerge-repo';
import {Repo} from '@automerge/automerge-repo';
import {RepoContext} from '@automerge/automerge-repo-react-hooks';
import {type TaskID, TaskStatus, type TunnelState} from '@mydoo/tasklens';
import {act, renderHook} from '@testing-library/react';
import type {ReactNode} from 'react';
import {afterEach, beforeEach, describe, expect, it} from 'vitest';

import {useDocument} from '../useDocument';
import {useSystemIntents} from './useSystemIntents';

describe('useSystemIntents', () => {
  let repo: Repo;

  beforeEach(() => {
    repo = new Repo({network: []});
    window.location.hash = '';
  });

  afterEach(() => {
    window.location.hash = '';
  });

  const wrapper = ({children}: {children: ReactNode}) => (
    <RepoContext.Provider value={repo}>{children}</RepoContext.Provider>
  );

  describe('refreshTaskList', () => {
    it('should acknowledge completed tasks', async () => {
      // 1. Setup Document
      const {result: docResult} = renderHook(() => useDocument(), {wrapper});
      const docUrl = docResult.current;

      const handle = await repo.find<TunnelState>(
        docUrl as unknown as DocumentId,
      );
      await handle.whenReady();

      // 2. Seed Data
      handle.change(d => {
        d.tasks['task1' as TaskID] = {
          id: 'task1' as TaskID,
          title: 'Pending',
          status: TaskStatus.Pending,
          isAcknowledged: false,
          childTaskIds: [],
          schedule: {type: 'Once', leadTime: 0},
          importance: 0.5,
          credits: 0,
          desiredCredits: 0,
          creditIncrement: 1,
          creditsTimestamp: 0,
          priorityTimestamp: 0,
          isSequential: false,
        };
        d.tasks['task2' as TaskID] = {
          id: 'task2' as TaskID,
          title: 'Done Unacked',
          status: TaskStatus.Done,
          isAcknowledged: false,
          childTaskIds: [],
          schedule: {type: 'Once', leadTime: 0},
          importance: 0.5,
          credits: 0,
          desiredCredits: 0,
          creditIncrement: 1,
          creditsTimestamp: 0,
          priorityTimestamp: 0,
          isSequential: false,
        };
        d.tasks['task3' as TaskID] = {
          id: 'task3' as TaskID,
          title: 'Done Acked',
          status: TaskStatus.Done,
          isAcknowledged: true,
          childTaskIds: [],
          schedule: {type: 'Once', leadTime: 0},
          importance: 0.5,
          credits: 0,
          desiredCredits: 0,
          creditIncrement: 1,
          creditsTimestamp: 0,
          priorityTimestamp: 0,
          isSequential: false,
        };
      });

      // 3. Setup Hook
      const {result} = renderHook(() => useSystemIntents(docUrl), {wrapper});

      // 4. Act
      act(() => {
        result.current.refreshTaskList();
      });

      // 5. Verify
      const doc = handle.doc();
      const t1 = doc.tasks['task1' as TaskID];
      const t2 = doc.tasks['task2' as TaskID];
      const t3 = doc.tasks['task3' as TaskID];

      expect(t1?.isAcknowledged).toBe(false);
      expect(t2?.isAcknowledged).toBe(true); // Changed!
      expect(t3?.isAcknowledged).toBe(true);
    });
  });
});
