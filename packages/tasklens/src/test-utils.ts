/**
 * Test utilities for TaskLens.
 *
 * This module provides helper functions for creating test fixtures and
 * initializing test state.
 */

import type {TunnelState} from './types';

/**
 * Creates an empty TunnelState document.
 *
 * This is useful for initializing test repositories with a valid document
 * structure that passes schema validation.
 *
 * @returns A minimal valid TunnelState with no tasks or places.
 *
 * @example
 * ```typescript
 * const repo = new Repo({network: []});
 * const handle = repo.create(createEmptyTunnelState());
 * ```
 */
export function createEmptyTunnelState(): TunnelState {
  return {
    tasks: {},
    rootTaskIds: [],
    places: {},
  };
}
