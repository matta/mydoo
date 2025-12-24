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
  /** View mode: 'tree' (desktop) or 'drill' (mobile). */
  viewMode: 'tree' | 'drill';
  /** Callback to open task editor. */
  onOpenEditor: (id: TaskID) => void;
}

/**
 * A recursive tree component that renders the task hierarchy.
 *
 * @remarks
 * - Iterates through the provided `nodes`.
 * - Renders a `TaskOutlineItem` for each node.
 * - Recursively renders itself for expanded children (only in tree mode or if expanded? Actually in drill mode we only show one level, so recursion handles child rendering if we were to support expansion in drill mode, but drill mode implies navigating *into* the view. Wait, existing code recursively renders if expanded. In drill mode, we might not have expanded logic?
 * NO: In drill mode, we show only the current level. If we want to show children, we drill down. So expansion might be disabled or hidden in drill mode. The requirement says "Drill-Down Mode: Show content of viewPath head only." which implies flat list of children.
 * But `displayRoots` in PlanViewContainer handles the "content of viewPath head".
 * So here we just iterate `nodes`. `expandedIds` might not be relevant for drill mode if we don't show chevrons.
 * But wait, if we are in drill mode, can we expand? "Icons: Hide Expand Chevron". So effectively flattened list at that level.
 * So the recursive call `isExpanded && ...` relies on `expandedIds`. If chevrons are hidden, we can't expand. So it effectively stays collapsed. That works.
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
              viewMode={viewMode}
              onOpenEditor={() => onOpenEditor(node.id)}
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
              />
            )}
          </div>
        );
      })}
    </Stack>
  );
}
