import type {DocHandle, DocumentId} from '@automerge/automerge-repo';
import {Repo} from '@automerge/automerge-repo';
import type {DocumentHandle, TunnelState} from '@mydoo/tasklens';
import {act, renderHook} from '@testing-library/react';
import {afterEach, beforeEach, describe, expect, it} from 'vitest';
import {createTestWrapper} from '../../test/setup';
import {useTaskIntents} from '../intents/use-task-intents';
import {useDocument} from '../use-document';
import {useBreadcrumbs} from './use-breadcrumbs';

describe('useBreadcrumbs', () => {
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

  it('should return empty array for root view', () => {
    const wrapper = createTestWrapper(repo);
    const {result} = renderHook(() => useBreadcrumbs(docUrl, undefined), {
      wrapper,
    });
    expect(result.current).toEqual([]);
  });

  it('should return path for nested task', async () => {
    // 1. Setup Data: Root -> Parent -> Child
    const wrapper = createTestWrapper(repo);
    const {result: intents} = renderHook(() => useTaskIntents(docUrl), {
      wrapper,
    });

    act(() => {
      intents.current.createTask('Parent');
    });
    const parentId = handle.doc().rootTaskIds[0];
    if (!parentId) throw new Error('Parent ID not found');

    act(() => {
      intents.current.createTask('Child', parentId);
    });
    const parent = handle.doc().tasks[parentId];
    if (!parent) throw new Error('Parent task not found');
    const childId = parent.childTaskIds[0];
    if (!childId) throw new Error('Child ID not found');

    // 2. Test Breadcrumbs when focused on Child
    // wrapper is already defined above
    const {result} = renderHook(() => useBreadcrumbs(docUrl, childId), {
      wrapper,
    });

    // Should be [Parent, Child]
    expect(result.current).toHaveLength(2);
    expect(result.current[0]?.title).toBe('Parent');
    expect(result.current[1]?.title).toBe('Child');
    expect(result.current[0]?.id).toBe(parentId);
    expect(result.current[1]?.id).toBe(childId);
  });
});
