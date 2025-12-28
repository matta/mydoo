import {type DocHandle, Repo} from '@automerge/automerge-repo';
import {
  createStore,
  type DocumentHandle,
  type TaskID,
  type TunnelState,
} from '@mydoo/tasklens';
import {act, renderHook, waitFor} from '@testing-library/react';
import {afterEach, beforeEach, describe, expect, it} from 'vitest';

import {createTestWrapper} from '../../test/setup';
import {useTaskIntents} from './use-task-intents';

describe('useTaskIntents (Move Interactions)', () => {
  let repo: Repo;
  let handle: DocHandle<TunnelState>;
  let docId: DocumentHandle;

  beforeEach(() => {
    repo = new Repo({network: []});
    window.location.hash = '';

    handle = repo.create<TunnelState>({
      tasks: {},
      rootTaskIds: [],
      places: {},
    });
    docId = handle.url as unknown as DocumentHandle;
  });

  afterEach(() => {
    window.location.hash = '';
  });

  it('should indent a task (become child of previous sibling)', async () => {
    const store = createStore();
    const wrapper = createTestWrapper(repo, store, docId);
    const {result} = renderHook(() => useTaskIntents(), {wrapper});

    // Setup: Root -> [Sibling, Target]
    let siblingId: TaskID;
    let targetId: TaskID;
    act(() => {
      siblingId = result.current.createTask('Sibling');
      targetId = result.current.createTask('Target');
    });

    // Wait for Redux to sync the tasks
    await waitFor(() => {
      const state = store.getState();
      if (!state.tasks.entities[siblingId])
        throw new Error('Sibling not in store');
      if (!state.tasks.entities[targetId])
        throw new Error('Target not in store');
    });

    // Indent target to be child of sibling
    act(() => {
      result.current.indentTask(targetId);
    });

    // Validate structure
    await waitFor(() => {
      const docAfter = handle.doc();
      const sibling = docAfter.tasks[siblingId];
      if (!sibling) throw new Error('Sibling task not found');
      expect(sibling.childTaskIds).toContain(targetId);

      const target = docAfter.tasks[targetId];
      if (!target) throw new Error('Target task not found');
      expect(target.parentId).toBe(siblingId);
    });
  });

  it('should outdent a task (become sibling of parent)', async () => {
    const store = createStore();
    const wrapper = createTestWrapper(repo, store, docId);
    const {result} = renderHook(() => useTaskIntents(), {wrapper});

    // Setup: Root -> Parent -> Child
    let parentId: TaskID;
    let childId: TaskID;
    act(() => {
      parentId = result.current.createTask('Parent');
    });

    // Wait for parent
    await waitFor(() => {
      const state = store.getState();
      if (!state.tasks.entities[parentId])
        throw new Error('Parent not in store');
    });

    act(() => {
      childId = result.current.createTask('Child', parentId);
    });

    // Wait for child
    await waitFor(() => {
      const state = store.getState();
      if (!state.tasks.entities[childId]) throw new Error('Child not in store');
    });

    act(() => {
      result.current.outdentTask(childId);
    });

    await waitFor(() => {
      const docAfter = handle.doc();
      expect(docAfter.rootTaskIds).toContain(childId);
      const child = docAfter.tasks[childId];
      if (!child) throw new Error('Child task not found');
      expect(child.parentId).toBeUndefined();
    });
  });

  it('should not indent if no previous sibling', async () => {
    const store = createStore();
    const wrapper = createTestWrapper(repo, store, docId);
    const {result} = renderHook(() => useTaskIntents(), {wrapper});

    let id: TaskID;
    act(() => {
      id = result.current.createTask('Solo');
    });

    // Wait for solo task
    await waitFor(() => {
      const state = store.getState();
      if (!state.tasks.entities[id]) throw new Error('Solo task not in store');
    });

    act(() => {
      result.current.indentTask(id);
    });

    await waitFor(() => {
      const doc = handle.doc();
      expect(doc.rootTaskIds).toHaveLength(1);
      const task = doc.tasks[id];
      if (!task) throw new Error('Task not found');
      expect(task.parentId).toBeUndefined();
    });
  });
});
