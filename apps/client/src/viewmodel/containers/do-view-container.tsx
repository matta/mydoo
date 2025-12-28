import {
  Button,
  Container,
  Group,
  LoadingOverlay,
  Stack,
  Title,
} from '@mantine/core';
import type {DocumentHandle, TaskID} from '@mydoo/tasklens';
import {IconRefresh} from '@tabler/icons-react';
import {useCallback} from 'react';

import {PriorityTaskList} from '../../components/composites/PriorityTaskList';
import {QuickAddInput} from '../../components/primitives/quick-add-input';
import {useSystemIntents} from '../intents/use-system-intents';
import {useTaskIntents} from '../intents/use-task-intents';
import {usePriorityList} from '../projections/use-priority-list';
import {useNavigationState} from '../ui/use-navigation-state';

export interface DoViewContainerProps {
  docUrl: DocumentHandle;
}

/**
 * DoViewContainer: Main container for the "Do" view (priority list).
 *
 * Orchestrates:
 * - Task list display via usePriorityList
 * - Quick task creation
 * - Task editing via global TaskEditorContainer (triggered via navigation state)
 */
export function DoViewContainer({docUrl}: DoViewContainerProps) {
  const {tasks, isLoading} = usePriorityList(docUrl);
  const {createTask, toggleTask} = useTaskIntents(docUrl);
  const {refreshTaskList} = useSystemIntents(docUrl);
  const {openEditModal} = useNavigationState();

  const handleToggle = useCallback(
    (id: TaskID) => {
      toggleTask(id);
    },
    [toggleTask],
  );

  const handleTitleTap = useCallback(
    (id: TaskID) => {
      openEditModal(id);
    },
    [openEditModal],
  );

  const handleCreate = useCallback(
    (text: string) => {
      createTask(text);
    },
    [createTask],
  );

  return (
    <Container pos="relative" py="xl" size="sm">
      <LoadingOverlay visible={isLoading} />

      <Stack gap="xl">
        <Group justify="space-between">
          <Title order={2}>Priorities</Title>
          <Button
            leftSection={<IconRefresh size={14} />}
            onClick={() => refreshTaskList()}
            size="xs"
            variant="light"
          >
            Refresh
          </Button>
        </Group>

        <QuickAddInput onAdd={handleCreate} />

        <PriorityTaskList
          onTitleTap={handleTitleTap}
          onToggle={handleToggle}
          tasks={tasks}
        />
      </Stack>
    </Container>
  );
}
