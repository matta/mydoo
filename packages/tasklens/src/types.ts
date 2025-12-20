export type TaskID = string;
export type PlaceID = string;

export const ANYWHERE_PLACE_ID: PlaceID = "Anywhere";

export enum TaskStatus {
  Pending = "Pending",
  Done = "Done",
  Deleted = "Deleted",
}

export interface ViewFilter {
  placeId?: PlaceID; // Can be 'All'
  includeClosed?: boolean;
}

export interface Context {
  currentTime: number;
  currentPlaceId?: PlaceID;
}

export interface Schedule {
  type: "Once" | "Recurring";
  dueDate: number | null; // Unix Timestamp
  leadTime: number; // Duration in ms
}

export interface Task {
  id: TaskID;
  title: string;
  parentId: TaskID | null;
  placeId: PlaceID | null;
  status: TaskStatus;
  importance: number;
  creditIncrement: number;
  credits: number;
  desiredCredits: number;
  creditsTimestamp: number; // Unix Timestamp (ms)
  priorityTimestamp: number; // Unix Timestamp (ms)
  schedule: Schedule;
  isSequential: boolean;

  // Computed properties (not stored directly)
  isContainer?: boolean;
  isPending?: boolean;
  isReady?: boolean;
  normalizedImportance?: number;
  effectiveCredits?: number;
  visibility?: boolean;
  priority?: number;
  feedbackFactor?: number;
  leadTimeFactor?: number;
}

export interface Place {
  id: PlaceID;
  hours: string; // Placeholder for Schedule/Bitmask definition
  includedPlaces: PlaceID[];
}

export interface TunnelState {
  tasks: Record<string, Task>;
  places: Record<string, Place>;
  nextTaskId: number;
  nextPlaceId: number;
  [key: string]: unknown;
}
