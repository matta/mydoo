import { Stack } from '@mantine/core';
import type { TaskID, TunnelNode } from '@mydoo/tasklens';

import { TaskOutlineItem } from '../../primitives/task-outline-item';

/**
 * Props for the OutlineTree component.
 */
export interface OutlineTreeProps {
  /** Map of expanded task IDs for O(1) lookup. */
  expandedIds: Set<TaskID>;
  /** The list of nodes to render at this level. */
  nodes: TunnelNode[];
  /** Callback when drill-down is requested. */
  onDrillDown: (id: TaskID) => void;
  /** Callback to toggle expansion of a node. */
  onExpandToggle: (id: TaskID) => void;
  /** Callback to toggle completion of a task. */
  onToggleCompletion: (id: TaskID) => void;
  /** Callback invoked when a task requests indentation. */
  onIndent: (id: TaskID) => void;
  /** Callback invoked when a task requests outdentation. */
  onOutdent: (id: TaskID) => void;
  /** The current indentation depth (handled internally by recursion). */
  depth?: number;
  /** View mode: 'tree' (desktop) or 'drill' (mobile). */
  viewMode: 'tree' | 'drill';
  /** Callback to open task editor. */
  onOpenEditor: (id: TaskID) => void;
  // Context Actions
  onAddSibling: (id: TaskID) => void;
  onAddChild: (id: TaskID) => void;
  onDelete: (id: TaskID) => void;
  /** The ID of the most recently created task (for highlight/scroll). */
  lastCreatedTaskId: TaskID | undefined;
  /** Whether the parent of these nodes is sequential. */
  isParentSequential?: boolean;
}

/**
 * A recursive tree component that renders the task hierarchy.
 *
 * @remarks
 * Renders a `TaskOutlineItem` for each node and recursively renders children
 * when expanded. In drill mode, expansion is effectively disabled (chevrons
 * hidden), so only the current level displays. The recursion still works
 * correctly because `expandedIds` won't contain drill-mode nodes.
 */
export function OutlineTree({
  expandedIds,
  nodes,
  onDrillDown,
  onExpandToggle,
  onToggleCompletion,
  onIndent,
  onOutdent,
  depth = 0,
  viewMode,
  onOpenEditor,
  onAddSibling,
  onAddChild,
  onDelete,
  lastCreatedTaskId,
  isParentSequential = false,
}: OutlineTreeProps) {
  if (nodes.length === 0) {
    return null;
  }

  return (
    <Stack gap={0}>
      {nodes.map((node, index) => {
        const isExpanded = expandedIds.has(node.id);

        return (
          <div key={node.id}>
            {/*
              PERF: Arrow functions create new references on each render.
              For large lists, consider memoizing TaskOutlineItem or using useCallback.
            */}
            <TaskOutlineItem
              depth={depth}
              isExpanded={isExpanded}
              node={node}
              onDrillDown={onDrillDown}
              onExpandToggle={onExpandToggle}
              onToggleCompletion={onToggleCompletion}
              onIndent={onIndent}
              onOutdent={onOutdent}
              viewMode={viewMode}
              onOpenEditor={onOpenEditor}
              onAddSibling={onAddSibling}
              onAddChild={onAddChild}
              onDelete={onDelete}
              isFlashTarget={node.id === lastCreatedTaskId}
              index={index}
              isSequentialChild={isParentSequential}
            />

            {isExpanded && node.children.length > 0 && (
              <OutlineTree
                nodes={node.children}
                expandedIds={expandedIds}
                depth={depth + 1}
                onDrillDown={onDrillDown}
                onExpandToggle={onExpandToggle}
                onToggleCompletion={onToggleCompletion}
                onIndent={onIndent}
                onOutdent={onOutdent}
                viewMode={viewMode}
                onOpenEditor={onOpenEditor}
                onAddSibling={onAddSibling}
                onAddChild={onAddChild}
                onDelete={onDelete}
                lastCreatedTaskId={lastCreatedTaskId}
                isParentSequential={node.isSequential}
              />
            )}
          </div>
        );
      })}
    </Stack>
  );
}
