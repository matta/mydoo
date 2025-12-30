import type {Repo} from '@automerge/automerge-repo';
import {
  asDocumentHandle,
  type DocumentHandle,
  initializeTunnelState,
  type TunnelState,
} from '@mydoo/tasklens';

/**
 * Creates a new initialized document without side effects.
 *
 * @param repo - The Automerge Repo instance.
 * @returns The DocumentHandle of the newly created document.
 */
export function createNewDocument(repo: Repo): DocumentHandle {
  const handle = repo.create<TunnelState>();
  handle.change(initializeTunnelState);
  return asDocumentHandle(handle.url);
}
