import {
  Box,
  Breadcrumbs,
  Button,
  Group,
  LoadingOverlay,
  Text,
} from '@mantine/core';
import type {DocumentHandle, TunnelNode} from '@mydoo/tasklens';
import {IconArrowLeft} from '@tabler/icons-react';
import {useMemo} from 'react';
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
  } = useNavigationState();
  const {toggleTask, indentTask, outdentTask} = useTaskIntents(docUrl);
  const breadcrumbs = useBreadcrumbs(docUrl, currentViewId);

  // Derive the subset of roots to display based on the current "drill-down" view.
  // If `currentViewId` is set, we traverse the tree to find that node and show its children.
  const displayRoots = useMemo(() => {
    if (!currentViewId) return roots;

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
  }, [roots, currentViewId]);

  if (isLoading) {
    return <LoadingOverlay visible />;
  }

  return (
    <Box
      p="md"
      style={{height: '100%', display: 'flex', flexDirection: 'column'}}
    >
      <Group justify="space-between" mb="md">
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
        </Group>
        <Group>
          <Button variant="default" size="xs" onClick={collapseAll}>
            Collapse All
          </Button>
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
        />

        {displayRoots.length === 0 && (
          <Text c="dimmed" fs="italic" ta="center" mt="xl">
            No tasks found.
          </Text>
        )}
      </Box>
    </Box>
  );
}
