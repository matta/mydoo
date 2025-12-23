import type {DocumentId} from '@automerge/automerge-repo';
import {Repo} from '@automerge/automerge-repo';
import {RepoContext} from '@automerge/automerge-repo-react-hooks';
import type {TunnelState} from '@mydoo/tasklens';
import {renderHook} from '@testing-library/react';
import type {ReactNode} from 'react';
import {afterEach, beforeEach, describe, expect, it} from 'vitest';

import {useDocument} from './useDocument';

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

  const wrapper = ({children}: {children: ReactNode}) => (
    <RepoContext.Provider value={repo}>{children}</RepoContext.Provider>
  );

  it('should create a new document if no hash is present', async () => {
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
    const existingId = 'test-doc-id';
    window.location.hash = `#${existingId}`;

    const {result} = renderHook(() => useDocument(), {wrapper});

    expect(result.current).toBe(existingId);
  });
});
