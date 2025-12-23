import {Box, Button, Group, LoadingOverlay, Text, Title} from '@mantine/core';
import type {DocumentHandle, TunnelNode} from '@mydoo/tasklens';
import {IconArrowLeft} from '@tabler/icons-react';
import {useMemo} from 'react';
import {useTaskIntents} from '../../../viewmodel/intents/useTaskIntents';
import {useTaskTree} from '../../../viewmodel/projections/useTaskTree';
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
 * Main container for the Plan View (hierarchical task list).
 *
 * Orchestrates:
 * - Fetching the task tree (`useTaskTree`).
 * - Managing navigation state (expansion, drill-down) via `useNavigationState`.
 * - Handling user intentions (`toggleTask` completion).
 * - Rendering the recursive `OutlineTree`.
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
  } = useNavigationState();
  const {toggleTask} = useTaskIntents(docUrl);

  // Filter the tree based on current drill-down view
  const displayRoots = useMemo(() => {
    if (!currentViewId) return roots;

    // Find the current view node in the existing tree
    // We need a helper to find a node by ID in the tree
    // Ideally useTaskTree could support this, or we just crawl here.
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

  const viewTitle = useMemo(() => {
    if (!currentViewId) return 'Plan';

    // Find title... similar crawl
    const findNode = (nodes: TunnelNode[]): TunnelNode | undefined => {
      for (const node of nodes) {
        if (node.id === currentViewId) return node;
        const found = findNode(node.children);
        if (found) return found;
      }
      return undefined;
    };

    return findNode(roots)?.title || 'Task Details';
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
          <Title order={3}>{viewTitle}</Title>
        </Group>
        <Group>
          <Button variant="default" size="xs" onClick={() => collapseAll()}>
            Collapse All
          </Button>
          {/* <Button variant="default" size="xs" onClick={() => expandAll()}>Expand All</Button> */}
        </Group>
      </Group>

      <Box style={{flex: 1, overflowY: 'auto'}}>
        <OutlineTree
          nodes={displayRoots}
          expandedIds={expandedIds}
          onDrillDown={pushView}
          onExpandToggle={toggleExpanded}
          onToggleCompletion={toggleTask}
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
