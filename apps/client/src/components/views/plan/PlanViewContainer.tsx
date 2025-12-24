import {
  Box,
  Breadcrumbs,
  Button,
  Group,
  LoadingOverlay,
  Text,
} from '@mantine/core';
import {useMediaQuery} from '@mantine/hooks';
import {
  type DocumentHandle,
  type TaskID,
  type TunnelNode,
  useTunnel,
} from '@mydoo/tasklens';
import {IconArrowLeft} from '@tabler/icons-react';
import {useEffect, useMemo} from 'react';
import {useTaskIntents} from '../../../viewmodel/intents/useTaskIntents';
import {useTaskTree} from '../../../viewmodel/projections/useTaskTree';
import {useBreadcrumbs} from '../../../viewmodel/ui/useBreadcrumbs';
import {useNavigationState} from '../../../viewmodel/ui/useNavigationState';
import {OutlineTree} from './OutlineTree';

/**
 * Props for PlanViewContainer.
 */
export interface PlanViewContainerProps {
  /** The Automerge document URL. */
  docUrl: DocumentHandle;
}

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
export function PlanViewContainer({docUrl}: PlanViewContainerProps) {
  const {roots, isLoading} = useTaskTree(docUrl);
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
  } = useNavigationState();
  const {doc} = useTunnel(docUrl);

  const {createTask, toggleTask, deleteTask, indentTask, outdentTask} =
    useTaskIntents(docUrl);
  const breadcrumbs = useBreadcrumbs(docUrl, currentViewId);

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

  // Responsive Breakpoint: 768px (sm) matches AppShell logic
  const isDesktop = useMediaQuery('(min-width: 768px)');
  const viewMode = isDesktop ? 'tree' : 'drill';

  // Strict Viewport Modes: Switching behavior
  useEffect(() => {
    if (isDesktop && viewPath.length > 0) {
      // Mobile -> Desktop: Reset viewPath to empty (show full tree)
      resetView();
    }
    // Desktop -> Mobile: Start at root for simplicity as per plan
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
    <Box
      p="md"
      style={{height: '100%', display: 'flex', flexDirection: 'column'}}
    >
      {/* Navigation Header - Only relevant in Drill-Down Mode (Mobile) or if depth > 0 */}
      <Group justify="space-between" mb="md">
        {!isDesktop && (
          <Group>
            {currentViewId && (
              <Button
                variant="subtle"
                leftSection={<IconArrowLeft size={16} />}
                onClick={popView}
                size="xs"
              >
                Back
              </Button>
            )}
            {/* Scrollable Breadcrumbs container for mobile */}
            <Box style={{overflowX: 'auto', maxWidth: '60vw'}}>
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
          </Group>
        )}
        <Group>
          {isDesktop && (
            <Button variant="default" size="xs" onClick={collapseAll}>
              Collapse All
            </Button>
          )}
        </Group>
      </Group>

      <Box style={{flex: 1, overflowY: 'auto'}}>
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
        />

        {displayRoots.length === 0 && (
          <Box ta="center" mt="xl">
            <Text c="dimmed" fs="italic" mb="md">
              No tasks found.
            </Text>
            <Button
              variant="light"
              onClick={() => {
                if (currentViewId) {
                  createTask('New Subtask', currentViewId);
                } else {
                  createTask('New Task');
                }
              }}
            >
              Add First Task
            </Button>
          </Box>
        )}
      </Box>
    </Box>
  );
}
