import type {DocumentId} from '@automerge/automerge-repo';
import {generateAutomergeUrl, Repo} from '@automerge/automerge-repo';
import type {TunnelState} from '@mydoo/tasklens';
import {renderHook} from '@testing-library/react';
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

    // Mock window.location
    window.location.hash = '';
  });

  afterEach(() => {
    window.location.hash = '';
  });

  it('should create a new document if no hash is present', async () => {
    const wrapper = createTestWrapper(repo);
    const {result} = renderHook(() => useDocument(), {wrapper});

    // It should generate a handle (opaque string)
    expect(result.current).toBeTruthy();
    expect(typeof result.current).toBe('string');

    // Hash should be updated
    expect(window.location.hash).toBe(`#${result.current}`);

    // Document should be initialized

    const handle = await repo.find<TunnelState>(
      result.current as unknown as DocumentId,
    );

    await handle.whenReady();

    const doc = handle.doc();

    expect(doc.nextTaskId).toBe(1);
  });

  it('should use existing hash if present', () => {
    const existingId = generateAutomergeUrl();
    window.location.hash = `#${existingId}`;

    const wrapper = createTestWrapper(repo);
    const {result} = renderHook(() => useDocument(), {wrapper});

    expect(result.current).toBe(existingId);
  });
});
