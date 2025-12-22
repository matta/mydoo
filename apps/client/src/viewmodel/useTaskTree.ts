import {type AnyDocumentId} from '@automerge/automerge-repo';
import {useTunnel} from '@mydoo/tasklens';

/**
 * ViewModel hook for accessing the task tree data.
 *
 * This hook projects the raw Automerge document into a tree structure
 * suitable for rendering. It specifically provides:
 * - The full task tree (nested TunnelNodes)
 * - The raw document state (TunnelState)
 * - Loading/Error states (implicitly via undefined return values)
 *
 * @param docUrl - The URL of the Automerge document.
 * @returns Object containing the task tree and raw document.
 */
export function useTaskTree(docUrl: AnyDocumentId) {
  const {tasks, doc} = useTunnel(docUrl);

  return {
    tasks,
    doc,
    isLoading: doc === undefined,
  };
}
