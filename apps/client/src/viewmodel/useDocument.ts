import type {AnyDocumentId} from '@automerge/automerge-repo';
import {useRepo} from '@automerge/automerge-repo-react-hooks';
import type {TunnelState} from '@mydoo/tasklens';
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
 * @returns {AnyDocumentId} The URL of the current Automerge document.
 */
export function useDocument() {
  const repo = useRepo();
  const [docUrl] = useState<AnyDocumentId>(() => {
    const hash = window.location.hash.slice(1);
    if (hash) return hash as AnyDocumentId;

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
    return url;
  });

  return docUrl;
}
