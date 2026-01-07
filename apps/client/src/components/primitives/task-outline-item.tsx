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
} from "@mantine/core";
import type { TaskID, TunnelNode } from "@mydoo/tasklens";
import {
  IconArrowRight,
  IconChevronDown,
  IconChevronRight,
  IconDots,
  IconPlus,
  IconTrash,
} from "@tabler/icons-react";
import { memo, useEffect, useRef } from "react";

import "./task-outline-item.css";

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
  onDrillDown: (id: TaskID) => void;
  /** Callback to toggle expansion state. */
  onExpandToggle: (id: TaskID) => void;
  /** Callback to toggle completion status. */
  onToggleCompletion: (id: TaskID) => void;
  /** Handler when the task is indented (e.g., via Tab key). */
  onIndent: (id: TaskID) => void;
  /** Handler when the task is outdented (e.g., via Shift+Tab key). */
  onOutdent: (id: TaskID) => void;
  /** View mode: 'tree' (desktop) or 'drill' (mobile). */
  viewMode: "tree" | "drill";
  /** Callback to open task editor. */
  onOpenEditor: (id: TaskID) => void;
  // Context Actions
  onAddSibling: (id: TaskID) => void;
  onAddChild: (id: TaskID) => void;
  onDelete: (id: TaskID) => void;
  /** Whether this task should flash to indicate it was just created/moved. */
  isFlashTarget?: boolean;
  /** 0-based index of the task in its parent's list. */
  index?: number;
  /** Whether the task is a child of a sequential project. */
  isSequentialChild?: boolean;
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
export const TaskOutlineItem = memo(function TaskOutlineItem({
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
  isFlashTarget,
  index = 0,
  isSequentialChild = false,
}: TaskOutlineItemProps) {
  const elementRef = useRef<HTMLDivElement>(null);
  const hasChildren = node.children.length > 0;

  useEffect(() => {
    if (isFlashTarget && elementRef.current) {
      // Scroll to view
      elementRef.current.scrollIntoView({
        behavior: "smooth",
        block: "center",
      });
    }
  }, [isFlashTarget]);

  /**
   * Intercepts Tab keys to trigger indent/outdent operations.
   * Prevents default browser focus traversal to allow structural editing.
   */
  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === "Tab") {
      event.preventDefault();
      if (event.shiftKey) {
        onOutdent(node.id);
      } else {
        onIndent(node.id);
      }
    } else if (event.key === "Enter") {
      onOpenEditor(node.id);
    }
  };

  const showChevron = viewMode === "tree";
  const showDrillDown = viewMode === "drill";

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
      className={`task-row ${isFlashTarget ? "flash-highlight" : ""}`}
      ref={elementRef}
    >
      {/* Task Actions Menu (Bullet replacement on Desktop, Context Menu on Mobile) */}
      <Menu
        position="bottom-start"
        shadow="md"
        width={200}
        withinPortal
        returnFocus={false}
      >
        <MenuTarget>
          <ActionIcon
            variant="transparent"
            size="sm"
            onClick={(e) => e.stopPropagation()}
            className="task-menu-trigger"
            aria-label="Task actions"
            data-testid="task-menu-trigger"
          >
            <IconDots style={{ width: rem(16), height: rem(16) }} />
          </ActionIcon>
        </MenuTarget>

        <MenuDropdown>
          <MenuItem
            leftSection={
              <IconPlus style={{ width: rem(14), height: rem(14) }} />
            }
            onClick={(e) => {
              e.stopPropagation();
              onAddSibling(node.id);
            }}
          >
            Add Sibling
          </MenuItem>
          <MenuItem
            leftSection={
              <IconPlus style={{ width: rem(14), height: rem(14) }} />
            }
            onClick={(e) => {
              e.stopPropagation();
              onAddChild(node.id);
            }}
          >
            Add Child
          </MenuItem>
          <MenuItem
            color="red"
            leftSection={
              <IconTrash style={{ width: rem(14), height: rem(14) }} />
            }
            onClick={(e) => {
              e.stopPropagation();
              onDelete(node.id);
            }}
          >
            Delete
          </MenuItem>
        </MenuDropdown>
      </Menu>

      {/* Expansion Chevron (Tree Mode Only) */}
      {showChevron && (
        <ActionIcon
          variant="subtle"
          size="sm"
          color="gray"
          onClick={(e) => {
            e.stopPropagation();
            onExpandToggle(node.id);
          }}
          aria-label="Toggle expansion"
          style={{
            opacity: hasChildren ? 1 : 0,
            pointerEvents: hasChildren ? "auto" : "none",
          }}
          // Used by E2E tests (fixtures.ts) to verify expansion state
          data-expanded={isExpanded}
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
        checked={node.status === "Done"}
        onChange={() => onToggleCompletion(node.id)}
        aria-label={`Complete ${node.title}`}
        size="xs"
        onClick={(e) => e.stopPropagation()}
      />

      {/* Task Title - Click opens Editor */}
      <Text
        size="sm"
        fw={500}
        {...(node.status === "Done" ? { c: "dimmed", td: "line-through" } : {})}
        style={{ flex: 1, cursor: "pointer" }}
        truncate
        onClick={() => onOpenEditor(node.id)}
      >
        {isSequentialChild && (
          <Text
            component="span"
            size="xs"
            c="dimmed"
            fw={700}
            style={{ marginRight: rem(8), verticalAlign: "middle" }}
          >
            {index + 1}.
          </Text>
        )}
        {node.title}
      </Text>

      {/* Drill Down Action (Mobile Only) - Always visible in drill mode to allow adding children */}
      {showDrillDown && (
        <ActionIcon
          variant="subtle"
          size="sm"
          color="gray"
          onClick={(e) => {
            e.stopPropagation();
            onDrillDown(node.id);
          }}
          aria-label="Drill down"
        >
          <IconArrowRight size={14} />
        </ActionIcon>
      )}
    </Group>
  );
});
