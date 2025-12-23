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
  /** Current depth level provided by recursion. */
  depth?: number;
}

/**
 * Recursive tree component for rendering the task hierarchy.
 *
 * Iterates through `nodes` and renders a `TaskOutlineItem` for each.
 * If a node is expanded and has children, it recursively renders another
 * `OutlineTree` for the children.
 */
export function OutlineTree({
  expandedIds,
  nodes,
  onDrillDown,
  onExpandToggle,
  onToggleCompletion,
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
            <TaskOutlineItem
              depth={depth}
              isExpanded={isExpanded}
              node={node}
              onDrillDown={() => onDrillDown(node.id)}
              onExpandToggle={() => onExpandToggle(node.id)}
              onToggleCompletion={() => onToggleCompletion(node.id)}
            />

            {isExpanded && node.children.length > 0 && (
              <OutlineTree
                nodes={node.children}
                expandedIds={expandedIds}
                depth={depth + 1}
                onDrillDown={onDrillDown}
                onExpandToggle={onExpandToggle}
                onToggleCompletion={onToggleCompletion}
              />
            )}
          </div>
        );
      })}
    </Stack>
  );
}
