/**
 * @mydoo/tasklens - Task management data model and operations.
 *
 * This package provides the core data structures and operations for managing
 * a hierarchical task list stored in an Automerge document.
 *
 * Main exports:
 * - **TunnelStore**: Class for managing task state in unit tests.
 * - **TunnelOps**: Pure functions for mutating task state.
 * - **useTunnel**: React hook for accessing task state in UI components.
 * - **Types**: TypeScript interfaces for Task, TunnelState, TunnelNode, etc.
 *
 * @example
 * ```typescript
 * // In a React component
 * import { useTunnel } from "@mydoo/tasklens";
 *
 * function MyComponent() {
 *   const { tasks, ops } = useTunnel(docUrl);
 *   return <button onClick={() => ops.add({ title: "New task" })}>Add</button>;
 * }
 * ```
 *
 * @example
 * ```typescript
 * // In a unit test
 * import { TunnelStore } from "@mydoo/tasklens";
 *
 * const store = new TunnelStore();
 * store.createTask({ title: "Test task" });
 * ```
 */
export { TunnelStore } from "./store";
export * from "./types";
export * as TunnelOps from "./ops";
export * from "./react";
