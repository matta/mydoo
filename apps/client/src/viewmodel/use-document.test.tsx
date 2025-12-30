import type {DocumentId} from '@automerge/automerge-repo';
import {generateAutomergeUrl, Repo} from '@automerge/automerge-repo';
import type {TunnelState} from '@mydoo/tasklens';
import {renderHook, waitFor} from '@testing-library/react';
import {afterEach, beforeEach, describe, expect, it} from 'vitest';

import {createTestWrapper} from '../test/setup';

import {useDocument} from './use-document';

describe('useDocument', () => {
  let repo: Repo;

  beforeEach(() => {
    // Setup repo
    repo = new Repo({
      network: [], // No network for tests
    });

    // Clear localStorage
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  it('should create a new document if no ID in storage', async () => {
    const wrapper = createTestWrapper(repo);
    const {result} = renderHook(() => useDocument(), {wrapper});

    // Wait for effect to create document and update state
    await waitFor(() => {
      expect(result.current).toBeTruthy();
    });

    // It should generate a handle (opaque string)
    expect(typeof result.current).toBe('string');

    // Storage should be updated
    expect(localStorage.getItem('mydoo:doc_id')).toBe(result.current);

    // Document should be initialized
    const handle = await repo.find<TunnelState>(
      result.current as unknown as DocumentId,
    );

    await handle.whenReady();
    const doc = handle.doc();
    expect(doc.nextTaskId).toBe(1);
  });

  it('should use existing ID from storage if present', async () => {
    const existingId = generateAutomergeUrl();
    localStorage.setItem('mydoo:doc_id', existingId);

    const wrapper = createTestWrapper(repo);
    const {result} = renderHook(() => useDocument(), {wrapper});

    // Even if it's synchronous in this branch, it's safer to be consistent
    await waitFor(() => {
      expect(result.current).toBe(existingId);
    });
  });
});
