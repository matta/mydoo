import {
  type AutomergeUrl,
  isValidAutomergeUrl,
} from '@automerge/automerge-repo';
import { useRepo } from '@automerge/automerge-repo-react-hooks';
import { useEffect, useState } from 'react';
import { createNewDocument } from './document-utils';

const STORAGE_KEY = 'mydoo:doc_id';

/**
 * useDocument Hook
 *
 * Manages the lifecycle of the primary Automerge document for the application.
 *
 * Behavior:
 * 1. Checks `localStorage` for a persisted Document ID.
 * 2. If a valid ID exists, returns it immediately.
 * 3. If no ID exists (first run), creates a new initialized document via the Repo.
 * 4. Persists the new Document ID to `localStorage` for future sessions.
 *
 * @returns {AutomergeUrl | undefined} The handle (URL) of the current document.
 *
 * **Return Value States:**
 * - `AutomergeUrl`: The system found a valid ID in `localStorage` or finished creating a new one.
 * - `undefined`: The system is in the **Bootstrap Phase**. No ID was found in storage.
 *
 * **The Bootstrap Mechanism:**
 * When `undefined` is returned, a side-effect (`useEffect`) immediately triggers:
 * 1. Calls `repo.create<TunnelState>()` to generate a new UUID.
 * 2. Applies the default schema (empty `tasks` and `places` maps) via `initializeTunnelState`.
 * 3. Persists the new ID to `localStorage`.
 * 4. Updates state to trigger a re-render with the valid `AutomergeUrl`.
 *
 * **Why is initialization async?**
 * Why not create the document synchronously in `useState`?
 * - **React Purity:** State initializers must be pure. Creating a document mutates the
 *   Automerge Repo and `localStorage`, which are side effects.
 * - **Concurrent Mode:** In valid React code, the initializer might run multiple times
 *   before a commit, potentially creating orphaned documents if done synchronously.
 * - **Separation:** Reading from storage is pure (no mutations), so it's safe
 *   in the initializer. Creating new resources is impure (mutates the Repo),
 *   so it must live in a `useEffect`.
 */
export function useDocument(): AutomergeUrl | undefined {
  const repo = useRepo();
  const [docUrl, setDocUrl] = useState<AutomergeUrl | undefined>(() => {
    // Initial state setup (runs once)
    const storedId = localStorage.getItem(STORAGE_KEY);

    if (storedId && isValidAutomergeUrl(storedId)) {
      return storedId as AutomergeUrl;
    }

    // Return undefined to indicate initialization needed
    return undefined;
  });

  useEffect(() => {
    if (!docUrl) {
      const handle = createNewDocument(repo);
      localStorage.setItem(STORAGE_KEY, handle);
      setDocUrl(handle);
    }
  }, [docUrl, repo]);

  return docUrl;
}
