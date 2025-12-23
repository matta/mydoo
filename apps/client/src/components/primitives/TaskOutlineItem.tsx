import {ActionIcon, Checkbox, Group, Text} from '@mantine/core';
import type {TunnelNode} from '@mydoo/tasklens';
import {
  IconArrowRight,
  IconChevronDown,
  IconChevronRight,
} from '@tabler/icons-react';

/**
 * Props for the TaskOutlineItem component.
 */
export interface TaskOutlineItemProps {
  /** The nesting depth (0-based) for indentation. */
  depth: number;
  /** Whether the node is currently expanded. */
  isExpanded: boolean;
  /** The task node to display. */
  node: TunnelNode;
  /** Callback for drill-down navigation (mobile/focus). */
  onDrillDown: () => void;
  /** Callback to toggle expansion state. */
  onExpandToggle: () => void;
  /** Callback to toggle completion status. */
  onToggleCompletion: () => void;
  /** Handler when the task is indented (e.g., via Tab key). */
  onIndent: () => void;
  /** Handler when the task is outdented (e.g., via Shift+Tab key). */
  onOutdent: () => void;
}

/**
 * Renders a single row in the hierarchical task outline.
 *
 * @remarks
 * This component is responsible for:
 * - Visualizing tree depth via padding.
 * - Handling expansion toggling for nodes with children.
 * - Providing direct action controls (completion, drill-down).
 * - Intercepting keyboard navigation (Tab/Shift+Tab) to trigger structural changes.
 */
export function TaskOutlineItem({
  depth,
  isExpanded,
  node,
  onDrillDown,
  onExpandToggle,
  onToggleCompletion,
  onIndent,
  onOutdent,
}: TaskOutlineItemProps) {
  const hasChildren = node.children.length > 0;

  /**
   * Intercepts Tab keys to trigger indent/outdent operations.
   * Prevents default browser focus traversal to allow structural editing.
   */
  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Tab') {
      event.preventDefault();
      if (event.shiftKey) {
        onOutdent();
      } else {
        onIndent();
      }
    }
  };

  return (
    <Group
      wrap="nowrap"
      align="center"
      data-testid="task-item"
      style={{
        paddingLeft: `calc(${depth} * var(--mantine-spacing-md))`,
        paddingTop: 4,
        paddingBottom: 4,
        minHeight: 36,
      }}
      onKeyDown={handleKeyDown}
      tabIndex={0} // Make row focusable for keyboard interaction
    >
      {/* Expansion Chevron */}
      <ActionIcon
        variant="subtle"
        size="sm"
        color="gray"
        onClick={onExpandToggle}
        aria-label="Toggle expansion"
        style={{
          opacity: hasChildren ? 1 : 0,
          pointerEvents: hasChildren ? 'auto' : 'none',
        }}
      >
        {isExpanded ? (
          <IconChevronDown size={14} />
        ) : (
          <IconChevronRight size={14} />
        )}
      </ActionIcon>

      {/* Completion Checkbox */}
      <Checkbox
        checked={node.status === 'Done'}
        onChange={onToggleCompletion}
        aria-label={`Complete ${node.title}`}
        size="xs"
      />

      {/* Task Title */}
      <Text
        size="sm"
        fw={500}
        {...(node.status === 'Done' ? {c: 'dimmed', td: 'line-through'} : {})}
        style={{flex: 1, cursor: 'default'}}
        truncate
      >
        {node.title}
      </Text>

      {/* Drill Down Action */}
      <ActionIcon
        variant="subtle"
        size="sm"
        color="gray"
        onClick={onDrillDown}
        aria-label="Focus view"
      >
        <IconArrowRight size={14} />
      </ActionIcon>
    </Group>
  );
}
