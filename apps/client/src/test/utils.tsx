import {Repo} from '@automerge/automerge-repo';
import {RepoContext} from '@automerge/automerge-repo-react-hooks';
import type {TunnelState} from '@mydoo/tasklens';
import type {ReactNode} from 'react';

export const createTestRepo = () => new Repo({network: []});

/**
 * Wrapper for testing hooks that require an Automerge Repo context.
 * Initializes a repo, creates a document with optional initial state,
 * and provides the context.
 */
export const withTestTunnel = (initialState: Partial<TunnelState> = {}) => {
  const repo = createTestRepo();

  // Create doc synchronously if possible, or we need to handle async setup
  // Since Repo is sync for local, we can create handle.
  const handle = repo.create<TunnelState>();
  handle.change(d => {
    Object.assign(d, {
      tasks: {},
      places: {},
      rootTaskIds: [],
      nextTaskId: 0,
      nextPlaceId: 0,
      ...initialState,
    });
  });

  // We return a React component wrapper, but we check if we need to return the docUrl too
  // The hook usually takes docUrl as arg.
  // So this helper might need to be used differently.
  // Standard pattern: return [wrapper, docUrl]

  // For simplicity in the test, let's just export the wrapper and let the test create the doc
  // if it needs specific setup, OR we simplify useSystemIntents test to assume docUrl is passed.

  const Wrapper = ({children}: {children: ReactNode}) => (
    <RepoContext.Provider value={repo}>{children}</RepoContext.Provider>
  );

  return Wrapper;
};

// Simplified setup for hooks
export async function setupHookTest(initialState: Partial<TunnelState> = {}) {
  const repo = createTestRepo();
  const handle = repo.create<TunnelState>();
  handle.change(d => {
    Object.assign(d, {
      tasks: {},
      places: {},
      rootTaskIds: [],
      nextTaskId: 0,
      nextPlaceId: 0,
      ...initialState,
    });
  });

  const wrapper = ({children}: {children: ReactNode}) => (
    <RepoContext.Provider value={repo}>{children}</RepoContext.Provider>
  );

  return {wrapper, docUrl: handle.url};
}
