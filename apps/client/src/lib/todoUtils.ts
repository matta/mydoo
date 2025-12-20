import type { TodoDoc, TodoList, TodoItem } from "./types";

/**
 * Navigate into a nested TodoList by following a path of IDs.
 * @param doc - The root document
 * @param path - Array of todo IDs representing the path
 * @returns The TodoList at that path, or null if path is invalid
 */
export function getListAtPath(doc: TodoDoc, path: string[]): TodoList | null {
  let current: TodoList = doc;
  for (const id of path) {
    if (!current.todos?.[id]) return null;
    current = current.todos[id].children;
  }
  return current;
}

/**
 * Check if a todo item can be marked as done.
 * A todo can only be marked done if ALL its children are already done.
 * @param item - The todo item to check
 * @returns true if the item can be marked done
 */
export function canMarkDone(item: TodoItem): boolean {
  if (!item.children?.todoOrder?.length) return true;

  return item.children.todoOrder.every((childId) => {
    const child = item.children.todos[childId];
    return child?.done;
  });
}

/**
 * Generate breadcrumb data for navigation.
 * @param doc - The root document
 * @param viewPath - Current path being viewed
 * @returns Array of breadcrumb objects with id, title, and path
 */
export function getBreadcrumbs(doc: TodoDoc, viewPath: string[]) {
  const crumbs = [{ id: "root", title: "Root", path: [] as string[] }];

  let currentPath: string[] = [];
  let currentList: TodoList = doc;

  for (const id of viewPath) {
    if (!currentList.todos?.[id]) break;
    const item = currentList.todos[id];
    currentPath = [...currentPath, id];
    crumbs.push({ id, title: item.title, path: currentPath });
    currentList = item.children;
  }
  return crumbs;
}

/**
 * Generate a unique ID for new todo items.
 * Uses crypto.randomUUID() for guaranteed uniqueness.
 */
export function generateId(): string {
  return crypto.randomUUID();
}
