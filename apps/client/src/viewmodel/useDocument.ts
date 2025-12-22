import {useRepo} from '@automerge/automerge-repo-react-hooks';
import type {DocumentHandle, TunnelState} from '@mydoo/tasklens';
import {useState} from 'react';

/**
 * useDocument Hook
 *
 * Managing the Automerge document lifecycle for the application.
 *
 * Responsibilities:
 * 1. Retrieves the document ID (URL) from the window location hash.
 * 2. If no hash exists, creates a new Automerge document with an initial empty state.
 * 3. Updates the window hash to match the new document URL.
 * 4. Returns the Document URL for use by other hooks (e.g. useTunnel).
 *
 * @returns {DocumentHandle} The URL of the current Automerge document.
 */
export function useDocument() {
  const repo = useRepo();
  const [docUrl] = useState<DocumentHandle>(() => {
    const hash = window.location.hash.slice(1);
    // Explicitly cast the hash string to our opaque type
    if (hash) return hash as unknown as DocumentHandle;

    const handle = repo.create<TunnelState>();
    handle.change(doc => {
      doc.tasks = {};
      doc.places = {};
      doc.rootTaskIds = [];
      doc.nextTaskId = 1;
      doc.nextPlaceId = 1;
    });
    const url = handle.url;
    window.location.hash = url;
    return url as unknown as DocumentHandle;
  });

  return docUrl;
}
