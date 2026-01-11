import type { AutomergeUrl, Repo } from "@automerge/automerge-repo";
import type { TunnelState } from "../types/persistence";

/**
 * Initializes a blank TunnelState document with default empty collections.
 *
 * @param doc - The mutable Automerge document proxy to initialize.
 */
export function initializeTunnelState(doc: TunnelState) {
  doc.tasks = {};
  doc.places = {};
  doc.rootTaskIds = [];
}

/**
 * Creates a new Automerge document initialized with the TunnelState schema.
 *
 * @param repo - The Automerge Repo instance.
 * @returns The AutomergeUrl of the newly created document.
 */
export function createTaskLensDoc(repo: Repo): AutomergeUrl {
  const handle = repo.create<TunnelState>();
  handle.change(initializeTunnelState);
  return handle.url;
}

/**
 * Checks if a document has been initialized with the TunnelState schema.
 *
 * @param doc - The document object to check.
 * @returns True if the document has valid TunnelState properties.
 */
export function isDocInitialized(doc: unknown): boolean {
  if (typeof doc !== "object" || doc === null) return false;
  // Check for key properties
  const state = doc as TunnelState;
  return typeof state.tasks === "object" && Array.isArray(state.rootTaskIds);
}
