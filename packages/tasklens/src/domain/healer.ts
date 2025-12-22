/**
 * Healer functions for maintaining data integrity.
 */
import type {TunnelState} from '../types';

/**
 * Deduplicates an array of strings (IDs).
 * @param ids Array of IDs
 * @returns New array with unique IDs, preserving order of first occurrence
 */
export function deduplicateArray<T extends string>(ids: T[]): T[] {
  const seen = new Set<string>();
  return ids.filter(id => {
    if (seen.has(id)) return false;
    seen.add(id);
    return true;
  });
}

/**
 * Heals the state by deduplicating task lists.
 * @param state The state to heal
 */
export function healState(state: TunnelState): void {
  state.rootTaskIds = deduplicateArray([...state.rootTaskIds]);
  for (const task of Object.values(state.tasks)) {
    task.childTaskIds = deduplicateArray([...task.childTaskIds]);
  }
}
