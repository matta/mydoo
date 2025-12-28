import type {DocHandle, DocumentId} from '@automerge/automerge-repo';
import {Repo} from '@automerge/automerge-repo';
import type {DocumentHandle, TunnelState} from '@mydoo/tasklens';
import {act, renderHook} from '@testing-library/react';
import {afterEach, beforeEach, describe, expect, it} from 'vitest';

import {createTestWrapper} from '../../test/setup';
import {useDocument} from '../use-document';
import {useTaskIntents} from './use-task-intents';

describe('useTaskIntents (Move Interactions)', () => {
  let repo: Repo;
  let docUrl: DocumentHandle;
  let handle: DocHandle<TunnelState>;

  beforeEach(async () => {
    repo = new Repo({network: []});
    window.location.hash = '';

    const wrapper = createTestWrapper(repo);

    const {result} = renderHook(() => useDocument(), {wrapper});
    docUrl = result.current;
    handle = await repo.find<TunnelState>(docUrl as unknown as DocumentId);
    await handle.whenReady();
  });

  afterEach(() => {
    window.location.hash = '';
  });

  it('should indent a task (become child of previous sibling)', async () => {
    const wrapper = createTestWrapper(repo);
    const {result} = renderHook(() => useTaskIntents(docUrl), {wrapper});

    // Setup: Root -> [Target, Sibling]
    // 1. Create Target -> [Target]
    // 2. Create Sibling -> [Sibling, Target]
    // Sibling is at 0, Target is at 1. Target's previous sibling is Sibling.

    act(() => {
      result.current.createTask('Sibling');
      result.current.createTask('Target');
    });

    const doc = handle.doc();
    const roots = doc.rootTaskIds;
    expect(roots).toHaveLength(2);
    // [Sibling, Target]
    const siblingId = doc.rootTaskIds[0];
    if (!siblingId) throw new Error('Sibling ID not found');
    const targetId = doc.rootTaskIds[1];
    if (!targetId) throw new Error('Target ID not found');

    const siblingTask = doc.tasks[siblingId];
    if (!siblingTask) throw new Error('Sibling task not found');
    expect(siblingTask.title).toBe('Sibling');

    const targetTask = doc.tasks[targetId];
    if (!targetTask) throw new Error('Target task not found');
    expect(targetTask.title).toBe('Target');

    // Indent target to be child of sibling
    await act(async () => {
      result.current.indentTask(targetId);
    });

    const docAfter = handle.doc();
    // Validate structure: Sibling should now have Target as child
    const sibling = docAfter.tasks[siblingId];
    if (!sibling) throw new Error('Sibling task not found');
    expect(sibling.childTaskIds).toContain(targetId);

    const target = docAfter.tasks[targetId];
    if (!target) throw new Error('Target task not found');
    expect(target.parentId).toBe(siblingId);
  });

  it('should outdent a task (become sibling of parent)', async () => {
    const wrapper = createTestWrapper(repo);
    const {result} = renderHook(() => useTaskIntents(docUrl), {wrapper});

    // Setup: Root -> Parent -> Child
    act(() => {
      result.current.createTask('Parent');
    });
    const parentId = handle.doc().rootTaskIds[0];
    if (!parentId) throw new Error('Parent ID not found');

    act(() => {
      result.current.createTask('Child', parentId);
    });

    const doc = handle.doc();
    const parent = doc.tasks[parentId];
    if (!parent) throw new Error('Parent task not found');

    const childId = parent.childTaskIds[0];
    if (!childId) throw new Error('Child ID not found');

    await act(async () => {
      result.current.outdentTask(childId);
    });

    const docAfter = handle.doc();
    // We relax order check if needed, but checking existence is key.
    const roots = docAfter.rootTaskIds;
    expect(roots).toContain(childId);

    const child = docAfter.tasks[childId];
    if (!child) throw new Error('Child task not found');
    expect(child.parentId).toBeUndefined();
  });

  it('should not indent if no previous sibling', async () => {
    const wrapper = createTestWrapper(repo);
    const {result} = renderHook(() => useTaskIntents(docUrl), {wrapper});

    act(() => {
      result.current.createTask('Solo');
    });
    const id = handle.doc().rootTaskIds[0];
    if (!id) throw new Error('ID not found');

    act(() => {
      result.current.indentTask(id);
    });

    const doc = handle.doc();
    if (doc.rootTaskIds.length === 0) throw new Error('No root tasks found');
    expect(doc.rootTaskIds).toHaveLength(1);

    // Check task existence
    if (!id) throw new Error('ID not found');
    const task = doc.tasks[id];
    if (!task) throw new Error('Task not found');
    expect(task.parentId).toBeUndefined();
  });
});
