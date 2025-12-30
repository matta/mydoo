import type {TunnelState} from '../types';

/**
 * Initializes a blank TunnelState document with default empty collections.
 *
 * @param doc - The mutable Automerge document proxy to initialize.
 */
export function initializeTunnelState(doc: TunnelState) {
  doc.tasks = {};
  doc.places = {};
  doc.rootTaskIds = [];
  doc.nextTaskId = 1; // Start IDs at 1
  doc.nextPlaceId = 1;
}
