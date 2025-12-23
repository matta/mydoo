import {Stack} from '@mantine/core';
import type {TaskID, TunnelNode} from '@mydoo/tasklens';

import {TaskOutlineItem} from '../../primitives/TaskOutlineItem';

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
}

/**
 * A recursive tree component that renders the task hierarchy.
 *
 * @remarks
 * - Iterates through the provided `nodes`.
 * - Renders a `TaskOutlineItem` for each node.
 * - Recursively renders itself for expanded children.
 * - Propagates all interaction callbacks (expansion, completion, movement) from the root to leaf nodes.
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
}: OutlineTreeProps) {
  if (nodes.length === 0) {
    return null;
  }

  return (
    <Stack gap={0}>
      {nodes.map(node => {
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
              onDrillDown={() => onDrillDown(node.id)}
              onExpandToggle={() => onExpandToggle(node.id)}
              onToggleCompletion={() => onToggleCompletion(node.id)}
              onIndent={() => onIndent(node.id)}
              onOutdent={() => onOutdent(node.id)}
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
              />
            )}
          </div>
        );
      })}
    </Stack>
  );
}
