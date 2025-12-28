import {
  ActionIcon,
  Box,
  Breadcrumbs,
  Button,
  Group,
  LoadingOverlay,
  Text,
} from '@mantine/core';
import {useMediaQuery} from '@mantine/hooks';
import type {RootState, TaskID, TunnelNode} from '@mydoo/tasklens';
import {IconArrowLeft, IconMenu, IconPlus} from '@tabler/icons-react';
import {useEffect, useMemo} from 'react';
import {useSelector} from 'react-redux';
import {useTaskIntents} from '../../../viewmodel/intents/use-task-intents';
import {useTaskTree} from '../../../viewmodel/projections/use-task-tree';
import {useBreadcrumbs} from '../../../viewmodel/ui/use-breadcrumbs';
import {useNavigationState} from '../../../viewmodel/ui/use-navigation-state';
import {OutlineTree} from './outline-tree';

/**
 * The primary container for the "Plan" view.
 *
 * @remarks
 * Orchestrates the hierarchical task display and management:
 * - **State Management**: Syncs with `useNavigationState` for expansion/collapse and drill-down history.
 * - **Data Projection**: Uses `useTaskTree` to transform raw Automerge data into a traversable tree structure.
 * - **User Intent**: Exposes operations for task modification (completion, structure changes via indent/outdent).
 * - **Navigation**: Calculates and renders breadcrumbs via `useBreadcrumbs`.
 * - **Responsiveness**: Switches between Tree Mode (Desktop) and Drill-Down Mode (Mobile).
 */
export function PlanViewContainer() {
  const {roots, isLoading} = useTaskTree();
  const {
    currentViewId,
    expandedIds,
    toggleExpanded,
    pushView,
    popView,
    collapseAll,
    resetView,
    setViewPath,
    openEditModal,
    openCreateModal,
    viewPath,
    lastCreatedTaskId,
  } = useNavigationState();
  const doc = useSelector((state: RootState) => state.tasks.lastDoc);

  const {toggleTask, deleteTask, indentTask, outdentTask} = useTaskIntents();
  const breadcrumbs = useBreadcrumbs(currentViewId);

  /**
   * Opens the create modal to add a sibling task after the specified task.
   * @param id - The reference task; the new sibling will be inserted after this task.
   */
  const handleAddSibling = (id: TaskID) => {
    if (!doc) return;
    const task = doc.tasks[id];
    if (task) {
      openCreateModal(task.parentId, id);
    }
  };

  /**
   * Opens the create modal to add a child task under the specified parent.
   * @param id - The parent task ID for the new child.
   */
  const handleAddChild = (id: TaskID) => {
    openCreateModal(id, undefined);
  };

  /**
   * Deletes the specified task and its descendants.
   * @param id - The task ID to delete.
   */
  const handleDelete = (id: TaskID) => {
    deleteTask(id);
  };

  /**
   * Handles creation from the Bottom Bar or Append Row.
   * @param position - 'start' (top) or 'end' (bottom).
   */
  const handleAddAtPosition = (position: 'start' | 'end') => {
    const parentId = currentViewId ?? undefined;
    openCreateModal(parentId, undefined, position);
  };

  // Responsive Breakpoint: 768px (sm) matches AppShell logic
  const isDesktop = useMediaQuery('(min-width: 768px)');
  const viewMode = isDesktop ? 'tree' : 'drill';

  // Strict Viewport Modes: Switching behavior
  useEffect(() => {
    if (isDesktop && viewPath.length > 0) {
      // Mobile -> Desktop: Reset viewPath to empty (show full tree)
      resetView();
    }
  }, [isDesktop, viewPath, resetView]);

  // Derive the subset of roots to display based on the current "drill-down" view.
  // If `currentViewId` is set, we traverse the tree to find that node and show its children.
  const displayRoots = useMemo(() => {
    // In Tree Mode (Desktop), always show full root list (drill-down is disabled)
    if (isDesktop || !currentViewId) {
      return roots;
    }

    const findNode = (nodes: TunnelNode[]): TunnelNode | undefined => {
      for (const node of nodes) {
        if (node.id === currentViewId) return node;
        const found = findNode(node.children);
        if (found) return found;
      }
      return undefined;
    };

    const target = findNode(roots);
    return target ? target.children : [];
  }, [roots, currentViewId, isDesktop]);

  if (isLoading) {
    return <LoadingOverlay visible />;
  }

  return (
    <Box style={{height: '100%', display: 'flex', flexDirection: 'column'}}>
      {/* Navigation Header - Top */}
      <Box p="md" pb="xs">
        <Group justify="space-between" mb="xs">
          {/* Breadcrumbs - Always visible if deep, but mainly for Mobile Drill-Down context */}
          {(!isDesktop || breadcrumbs.length > 0) && (
            <Box style={{overflowX: 'auto', flex: 1}}>
              <Breadcrumbs separator=">">
                <Button
                  variant="subtle"
                  size="xs"
                  onClick={resetView}
                  fw={breadcrumbs.length === 0 ? 'bold' : 'normal'}
                  c={breadcrumbs.length === 0 ? 'text' : 'dimmed'}
                  px={4}
                >
                  Plan
                </Button>
                {breadcrumbs.map((item, index) => {
                  const isLast = index === breadcrumbs.length - 1;
                  return (
                    <Button
                      key={item.id}
                      variant="subtle"
                      size="xs"
                      onClick={() => {
                        const newPath = breadcrumbs
                          .slice(0, index + 1)
                          .map(b => b.id);
                        setViewPath(newPath);
                      }}
                      fw={isLast ? 'bold' : 'normal'}
                      c={isLast ? 'text' : 'dimmed'}
                      px={4}
                    >
                      {item.title}
                    </Button>
                  );
                })}
              </Breadcrumbs>
            </Box>
          )}
          <Group>
            {isDesktop && (
              <Button variant="default" size="xs" onClick={collapseAll}>
                Collapse All
              </Button>
            )}
          </Group>
        </Group>
      </Box>

      {/* Main Content Area - Scrollable */}
      <Box style={{flex: 1, overflowY: 'auto'}} px="md">
        <OutlineTree
          nodes={displayRoots}
          expandedIds={expandedIds}
          onDrillDown={pushView}
          onExpandToggle={toggleExpanded}
          onToggleCompletion={toggleTask}
          onIndent={indentTask}
          onOutdent={outdentTask}
          viewMode={viewMode}
          onOpenEditor={openEditModal}
          onAddSibling={handleAddSibling}
          onAddChild={handleAddChild}
          onDelete={handleDelete}
          lastCreatedTaskId={lastCreatedTaskId}
        />

        {displayRoots.length === 0 && (
          <Box ta="center" mt="xl">
            <Text c="dimmed" fs="italic" mb="md">
              No tasks found.
            </Text>
            <Button
              variant="light"
              onClick={() => {
                handleAddAtPosition('end');
              }}
            >
              Add First Task
            </Button>
          </Box>
        )}

        {/* Append Row - Visible at bottom of list */}
        {displayRoots.length > 0 && (
          <Button
            variant="subtle"
            fullWidth
            h={48}
            mt="md"
            justify="center"
            onClick={() => handleAddAtPosition('end')}
            leftSection={<IconPlus size={16} />}
            c="dimmed"
            aria-label="Append Row"
            data-testid="append-row-button"
          >
            {/* Icon access via leftSection, empty text */}
          </Button>
        )}

        {/* Spacer for bottom bar on mobile */}
        {!isDesktop && <Box h={60} />}
      </Box>

      {/* Mobile Bottom Bar - Fixed */}
      {!isDesktop && (
        <Group
          justify="space-between"
          p="md"
          gap="0"
          style={{
            borderTop: '1px solid var(--mantine-color-default-border)',
            backgroundColor: 'var(--mantine-color-body)',
            position: 'sticky',
            bottom: 0,
            zIndex: 10,
          }}
        >
          {/* Hamburger (Disabled) */}
          <ActionIcon
            variant="subtle"
            disabled
            aria-label="Menu"
            size="lg"
            color="gray"
          >
            <IconMenu size={22} />
          </ActionIcon>

          {/* Up Level / Back */}
          <ActionIcon
            variant="subtle"
            onClick={popView}
            disabled={!currentViewId}
            aria-label="Up Level"
            size="lg"
            color="gray"
          >
            <IconArrowLeft size={22} />
          </ActionIcon>

          {/* Add at Top */}
          <ActionIcon
            variant="transparent"
            radius="xl"
            size="xl"
            onClick={() => handleAddAtPosition('start')}
            aria-label="Add Task at Top"
            style={{border: '1px solid var(--mantine-color-default-border)'}}
          >
            <IconPlus size={26} />
          </ActionIcon>

          {/* Empty Right Slot */}
          <Box w={40} />
        </Group>
      )}
    </Box>
  );
}
