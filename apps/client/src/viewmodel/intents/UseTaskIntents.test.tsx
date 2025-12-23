import type {DocumentId} from '@automerge/automerge-repo';
import {Repo} from '@automerge/automerge-repo';
import {RepoContext} from '@automerge/automerge-repo-react-hooks';
import type {TunnelState} from '@mydoo/tasklens';
import {act, renderHook} from '@testing-library/react';
import type {ReactNode} from 'react';
import {afterEach, beforeEach, describe, expect, it} from 'vitest';

import {useDocument} from '../useDocument';
import {useTaskIntents} from './useTaskIntents';

describe('useTaskIntents', () => {
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

  it('should create a task', async () => {
    // 1. Setup Document
    const {result: docResult} = renderHook(() => useDocument(), {wrapper});
    const docUrl = docResult.current;

    // Wait for doc to be ready
    const handle = await repo.find<TunnelState>(
      docUrl as unknown as DocumentId,
    );
    await handle.whenReady();

    // 2. Setup Intents Hook
    const {result} = renderHook(() => useTaskIntents(docUrl), {wrapper});

    // 3. Create Task
    act(() => {
      result.current.createTask('Buy Milk');
    });

    // 4. Verify in Repo
    const doc = handle.doc();
    const tasks = Object.values(doc.tasks);

    expect(tasks).toHaveLength(1);
    const createdTask = tasks[0];

    if (!createdTask) throw new Error('Task missing');

    expect(createdTask.title).toBe('Buy Milk');
    expect(createdTask.status).toBe('Pending');
  });

  it('should toggle task completion', async () => {
    // 1. Setup Document
    const {result: docResult} = renderHook(() => useDocument(), {wrapper});
    const docUrl = docResult.current;

    const handle = await repo.find<TunnelState>(
      docUrl as unknown as DocumentId,
    );
    await handle.whenReady();

    // 2. Setup Intents Hook
    const {result} = renderHook(() => useTaskIntents(docUrl), {wrapper});

    // 3. Create Task
    act(() => {
      result.current.createTask('Walk Dog');
    });

    const docBefore = handle.doc();
    const task = Object.values(docBefore.tasks)[0];

    if (!task) throw new Error('Task not found');
    expect(task.status).toBe('Pending');
    const taskId = task.id;

    // 4. Toggle Completion
    act(() => {
      result.current.toggleTaskCompletion(taskId);
    });

    // 5. Verify
    const docAfter = handle.doc();

    const taskAfter = docAfter.tasks[taskId];

    if (!taskAfter) throw new Error('Task missing in update');
    expect(taskAfter.status).toBe('Done');

    // 6. Toggle Back
    act(() => {
      result.current.toggleTaskCompletion(taskId);
    });

    const docFinal = handle.doc();

    const taskFinal = docFinal.tasks[taskId];

    if (!taskFinal) throw new Error('Task missing in final');
    expect(taskFinal.status).toBe('Pending');
  });
});
