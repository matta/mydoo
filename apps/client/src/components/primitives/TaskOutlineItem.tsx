import {
  ActionIcon,
  Checkbox,
  Group,
  Menu,
  MenuDropdown,
  MenuItem,
  MenuTarget,
  rem,
  Text,
} from '@mantine/core';
import type {TunnelNode} from '@mydoo/tasklens';
import {
  IconArrowRight,
  IconChevronDown,
  IconChevronRight,
  IconDots,
  IconPlus,
  IconTrash,
} from '@tabler/icons-react';

import './TaskOutlineItem.css';

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
  /** View mode: 'tree' (desktop) or 'drill' (mobile). */
  viewMode: 'tree' | 'drill';
  /** Callback to open task editor. */
  onOpenEditor: () => void;
  // Context Actions
  onAddSibling: () => void;
  onAddChild: () => void;
  onDelete: () => void;
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
  viewMode,
  onOpenEditor,
  onAddSibling,
  onAddChild,
  onDelete,
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
    } else if (event.key === 'Enter') {
      onOpenEditor();
    }
  };

  const showChevron = viewMode === 'tree';
  const showDrillDown = viewMode === 'drill';

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
      className="task-row"
    >
      {/* Desktop Hover Menu (replaces Bullet) */}
      {viewMode === 'tree' && (
        <Menu position="bottom-start" shadow="md" width={200} withinPortal>
          <MenuTarget>
            <ActionIcon
              variant="transparent"
              size="sm"
              onClick={e => e.stopPropagation()}
              className="task-menu-trigger"
              aria-label="Task actions"
              data-testid="task-menu-trigger"
            >
              <IconDots style={{width: rem(16), height: rem(16)}} />
            </ActionIcon>
          </MenuTarget>

          <MenuDropdown>
            <MenuItem
              leftSection={
                <IconPlus style={{width: rem(14), height: rem(14)}} />
              }
              onClick={e => {
                e.stopPropagation();
                onAddSibling();
              }}
            >
              Add Sibling
            </MenuItem>
            <MenuItem
              leftSection={
                <IconPlus style={{width: rem(14), height: rem(14)}} />
              }
              onClick={e => {
                e.stopPropagation();
                onAddChild();
              }}
            >
              Add Child
            </MenuItem>
            <MenuItem
              color="red"
              leftSection={
                <IconTrash style={{width: rem(14), height: rem(14)}} />
              }
              onClick={e => {
                e.stopPropagation();
                onDelete();
              }}
            >
              Delete
            </MenuItem>
          </MenuDropdown>
        </Menu>
      )}

      {/* Expansion Chevron (Tree Mode Only) */}
      {showChevron && (
        <ActionIcon
          variant="subtle"
          size="sm"
          color="gray"
          onClick={e => {
            e.stopPropagation();
            onExpandToggle();
          }}
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
      )}

      {/* Completion Checkbox */}
      <Checkbox
        checked={node.status === 'Done'}
        onChange={onToggleCompletion}
        aria-label={`Complete ${node.title}`}
        size="xs"
        onClick={e => e.stopPropagation()}
      />

      {/* Task Title - Click opens Editor */}
      <Text
        size="sm"
        fw={500}
        {...(node.status === 'Done' ? {c: 'dimmed', td: 'line-through'} : {})}
        style={{flex: 1, cursor: 'pointer'}}
        truncate
        onClick={onOpenEditor}
      >
        {node.title}
      </Text>

      {/* Drill Down Action (Mobile Only) */}
      {showDrillDown && hasChildren && (
        <ActionIcon
          variant="subtle"
          size="sm"
          color="gray"
          onClick={e => {
            e.stopPropagation();
            onDrillDown();
          }}
          aria-label="Drill down"
        >
          <IconArrowRight size={14} />
        </ActionIcon>
      )}
    </Group>
  );
}
