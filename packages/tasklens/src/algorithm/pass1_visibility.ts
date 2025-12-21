import * as Automerge from "@automerge/automerge";
import {
  type Task,
  type Place,
  type TunnelState,
  type ViewFilter,
  type TaskID,
  ANYWHERE_PLACE_ID,
} from "../../src/types";

import type { OpenHours } from "../../specs/compliance/schemas/test_case";

// Helper to get place from a given doc state
function _getPlaceFromDoc(
  docState: TunnelState,
  id: TaskID,
): Place | undefined {
  return docState.places[id];
}

// Helper to ensure exhaustive switch case matching
function assertUnreachable(x: never): never {
  // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
  throw new Error(`Unexpected value: ${x}`);
}

// Helper to check if a place is open
function _isPlaceOpen(place: Place, currentTime: number): boolean {
  if (!place.hours) {
    return true; // Assume open if no hours specified
  }

  const openHours = JSON.parse(place.hours) as OpenHours;

  switch (openHours.mode) {
    case "always_open":
      return true;
    case "always_closed":
      return false;
    case "custom": {
      // Custom schedule check
      if (!openHours.schedule) {
        return false; // Should not happen if valid, but safe fallback
      }
      const date = new Date(currentTime);
      const dayOfWeek = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][
        date.getUTCDay()
      ];
      if (!dayOfWeek) return false;
      const currentHour = date.getUTCHours();
      const currentMinute = date.getUTCMinutes();

      const daySchedule = openHours.schedule[dayOfWeek];
      if (daySchedule) {
        for (const range of daySchedule) {
          const [start, end] = range.split("-").map((time) => {
            const [h, m] = time.split(":").map(Number);
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
 * Pass 1: Contextual Visibility
 * Filters tasks by Physical Context and Time.
 * Updates the `visibility` property of each task.
 * @param doc The current Automerge document state (mutable proxy).
 * @param tasks All tasks in the document.
 * @param viewFilter The active view filter from the user.
 * @param currentTime The current timestamp in milliseconds.
 */
export function pass1ContextualVisibility(
  doc: Automerge.Doc<TunnelState>,
  viewFilter: ViewFilter,
  currentTime: number,
): void {
  for (const taskId in doc.tasks) {
    const task = doc.tasks[taskId];

    // 1. Resolve Effective Place
    let effectivePlaceId: TaskID | null = task?.placeId ?? null;
    if (effectivePlaceId === null && task?.parentId !== null) {
      let currentParent: Task | undefined = task?.parentId
        ? doc.tasks[task.parentId]
        : undefined;
      while (currentParent?.placeId === null) {
        currentParent = currentParent.parentId
          ? doc.tasks[currentParent.parentId]
          : undefined;
      }
      if (currentParent) {
        effectivePlaceId = currentParent.placeId;
      } else {
        effectivePlaceId = ANYWHERE_PLACE_ID;
      }
    }
    effectivePlaceId ??= ANYWHERE_PLACE_ID; // Root task with no place defaults to Anywhere

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
    if (viewFilter.placeId === "All") {
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
    if (task) {
      task.visibility = isOpen && filterMatch;
    }
  }
}
