/**
 * Utility functions for navigating and querying the task tree.
 *
 * These functions operate on `TunnelNode` objects, which represent tasks
 * with their children already resolved into a tree structure.
 */
import {type TaskID, TaskStatus, type TunnelNode} from '@mydoo/tasklens';

/**
 * Navigates into a nested task list by following a path of task IDs.
 *
 * This function traverses the task tree by following each ID in the path
 * and returning the children of the final task. It's used to implement
 * "drill-down" navigation in the UI.
 *
 * @param tasks - The root list of tasks (top-level TunnelNodes).
 * @param path - An array of task IDs representing the navigation path.
 *               An empty array returns the root list.
 * @returns The list of children at the specified path, or null if any ID
 *          in the path is not found (invalid path).
 *
 * @example
 * // Given: root -> "1" (Work) -> "2" (Project A)
 * getListAtPath(tasks, [])      // returns root tasks
 * getListAtPath(tasks, ["1"])   // returns children of "Work"
 * getListAtPath(tasks, ["1", "2"]) // returns children of "Project A"
 * getListAtPath(tasks, ["999"]) // returns null (ID not found)
 */
export function getListAtPath(
  tasks: TunnelNode[],
  path: TaskID[],
): TunnelNode[] | null {
  let currentList = tasks;
  for (const id of path) {
    const item = currentList.find(t => t.id === id);
    if (!item) return null;
    currentList = item.children;
  }
  return currentList;
}

/**
 * Checks whether a task can be marked as completed.
 *
 * A task can only be marked "Done" if all of its children are already
 * completed (Done) or removed (Deleted). This prevents marking a parent
 * task as done while work remains on subtasks.
 *
 * @param item - The task to check.
 * @returns true if the task has no children, or all children are Done/Deleted.
 *
 * @example
 * // Task with no children
 * canMarkDone({ ...task, children: [] }) // true
 *
 * // Task with all children done
 * canMarkDone({ ...task, children: [doneTask, deletedTask] }) // true
 *
 * // Task with pending children
 * canMarkDone({ ...task, children: [pendingTask] }) // false
 */
export function canMarkDone(item: TunnelNode): boolean {
  if (item.children.length === 0) return true;

  return item.children.every(
    child =>
      child.status === TaskStatus.Done || child.status === TaskStatus.Deleted,
  );
}

/**
 * A breadcrumb item representing either the root level or a task in the hierarchy.
 *
 * This discriminated union ensures type safety when handling breadcrumbs:
 * - Root items have `type: 'root'` and no task ID.
 * - Task items have `type: 'task'` with a valid TaskID.
 */
export type BreadcrumbItem =
  | {path: TaskID[]; title: string; type: 'root'}
  | {id: TaskID; path: TaskID[]; title: string; type: 'task'};

/**
 * Generates breadcrumb navigation data for the current view path.
 *
 * Breadcrumbs show the user's current location in the task hierarchy,
 * allowing them to navigate back to parent tasks or the root.
 *
 * @param tasks - The root list of tasks.
 * @param viewPath - The current navigation path (array of task IDs).
 * @returns An array of BreadcrumbItem objects for navigation.
 *
 * @example
 * // Current path: ["1", "2"] (Root -> Work -> Project A)
 * getBreadcrumbs(tasks, ["1", "2"])
 * // Returns:
 * // [
 * //   { type: "root", title: "Root", path: [] },
 * //   { type: "task", id: "1", title: "Work", path: ["1"] },
 * //   { type: "task", id: "2", title: "Project A", path: ["1", "2"] }
 * // ]
 */
export function getBreadcrumbs(
  tasks: TunnelNode[],
  viewPath: TaskID[],
): BreadcrumbItem[] {
  const crumbs: BreadcrumbItem[] = [{type: 'root', title: 'Root', path: []}];

  let currentPath: TaskID[] = [];
  let currentList = tasks;

  for (const id of viewPath) {
    const item = currentList.find(t => t.id === id);
    if (!item) break;
    currentPath = [...currentPath, id];
    crumbs.push({type: 'task', id, title: item.title, path: currentPath});
    currentList = item.children;
  }
  return crumbs;
}
