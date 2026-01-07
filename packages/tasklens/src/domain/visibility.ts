/**
 * Contextual Visibility Algorithm
 *
 * This module implements the contextual visibility rules for the task prioritization algorithm.
 * It determines which tasks should be visible based on the user's current
 * physical context (location) and time.
 *
 * The algorithm:
 * 1. Resolves each task's "effective place" by walking up the parent chain
 *    until a place is found (tasks inherit their place if not set).
 * 2. Checks if the effective place is currently "open" based on its schedule.
 * 3. Checks if the task matches the current view filter (selected location).
 * 4. Sets `task.visibility = isOpen && filterMatch`.
 *
 * This pass runs before priority calculations, so only visible tasks are
 * considered for ranking.
 */
import {
  ANYWHERE_PLACE_ID,
  type EnrichedTask,
  type OpenHours,
  type Place,
  type PlaceID,
  type TunnelState,
  type ViewFilter,
} from '../../src/types';

// Helper to get place from a given doc state
function _getPlaceFromDoc(
  docState: TunnelState,
  id: PlaceID,
): Place | undefined {
  return docState.places[id];
}

// Helper to ensure exhaustive switch case matching
function assertUnreachable(_: never): never {
  throw new Error('Unexpected value in exhaustive check');
}

// Helper to check if a place is open
function _isPlaceOpen(place: Place, currentTime: number): boolean {
  if (!place.hours) {
    return true; // Assume open if no hours specified
  }

  const openHours = JSON.parse(place.hours) as OpenHours;

  switch (openHours.mode) {
    case 'always_open':
      return true;
    case 'always_closed':
      return false;
    case 'custom': {
      // Custom schedule check
      if (!openHours.schedule) {
        return false; // Should not happen if valid, but safe fallback
      }
      const date = new Date(currentTime);
      const dayOfWeek = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'][
        date.getUTCDay()
      ];
      if (!dayOfWeek) return false;
      const currentHour = date.getUTCHours();
      const currentMinute = date.getUTCMinutes();

      const daySchedule = openHours.schedule[dayOfWeek];
      if (daySchedule) {
        for (const range of daySchedule) {
          const [start, end] = range.split('-').map((time) => {
            const [h, m] = time.split(':').map(Number);
            return (h ?? 0) * 60 + (m ?? 0); // Convert to minutes past midnight
          });
          const currentTimeInMinutes = currentHour * 60 + currentMinute;
          if (
            start !== undefined &&
            end !== undefined &&
            currentTimeInMinutes >= start &&
            currentTimeInMinutes < end
          ) {
            return true;
          }
        }
      }
      return false;
    }
    default:
      return assertUnreachable(openHours.mode);
  }
}

/**
 * Contextual Visibility
 * Filters tasks by Physical Context and Time.
 * Updates the `visibility` property of each task.
 * @param doc The current Automerge document state (mutable proxy).
 * @param tasks All tasks in the document (Mutable EnrichedTasks).
 * @param viewFilter The active view filter from the user.
 * @param currentTime The current timestamp in milliseconds.
 */
export function calculateContextualVisibility(
  doc: TunnelState,
  tasks: EnrichedTask[],
  viewFilter: ViewFilter,
  currentTime: number,
): void {
  for (const task of tasks) {
    // 1. Resolve Effective Place
    // Note: PlaceID follows the "Copy-on-Create" strategy. Tasks own their
    // assigned placeId and do not inherit from ancestors at runtime.
    const effectivePlaceId = task.placeId ?? ANYWHERE_PLACE_ID;

    // Write back the resolved place to the enriched task so it can be used by the UI/Tests
    task.placeId = effectivePlaceId;

    const effectivePlace = effectivePlaceId
      ? _getPlaceFromDoc(doc, effectivePlaceId)
      : undefined;

    // 2. Hours Check (IsOpen)
    let isOpen = false;
    if (effectivePlaceId === ANYWHERE_PLACE_ID) {
      isOpen = true; // Anywhere is always open
    } else if (effectivePlace) {
      isOpen = _isPlaceOpen(effectivePlace, currentTime);
    } else {
      isOpen = false;
    }

    // 3. Place Match
    let filterMatch = false;
    // View filter can be 'All', 'Anywhere' or a specific place ID
    if (viewFilter.placeId === 'All') {
      filterMatch = true;
    } else if (effectivePlaceId === ANYWHERE_PLACE_ID) {
      filterMatch = true; // Anywhere tasks always appear in any filter (universal inclusion)
    } else if (viewFilter.placeId === effectivePlaceId) {
      filterMatch = true; // Direct match
    } else if (
      // Check if the task's place is included in the filter's place
      viewFilter.placeId &&
      viewFilter.placeId !== ANYWHERE_PLACE_ID
    ) {
      const filterPlace = _getPlaceFromDoc(doc, viewFilter.placeId);
      if (filterPlace?.includedPlaces.includes(effectivePlaceId)) {
        filterMatch = true;
      }
    }

    // Final Visibility for this pass
    task.visibility = isOpen && filterMatch;
  }
}
