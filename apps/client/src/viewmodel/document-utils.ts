import type { AutomergeUrl, Repo } from '@automerge/automerge-repo';
import { initializeTunnelState, type TunnelState } from '@mydoo/tasklens';

/**
 * Creates a new initialized document without side effects.
 *
 * @param repo - The Automerge Repo instance.
 * @returns The AutomergeUrl of the newly created document.
 */
export function createNewDocument(repo: Repo): AutomergeUrl {
  const handle = repo.create<TunnelState>();
  handle.change(initializeTunnelState);
  return handle.url;
}
