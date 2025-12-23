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

import {PriorityTaskList} from '../../components/composites/PriorityTaskList';
import {QuickAddInput} from '../../components/primitives/QuickAddInput';
import {useSystemIntents} from '../intents/useSystemIntents';
import {useTaskIntents} from '../intents/useTaskIntents';
import {usePriorityList} from '../projections/usePriorityList';

export interface DoViewContainerProps {
  docUrl: DocumentHandle;
}

export function DoViewContainer({docUrl}: DoViewContainerProps) {
  const {tasks, isLoading} = usePriorityList(docUrl);
  const {createTask, toggleTaskCompletion} = useTaskIntents(docUrl);
  const {refreshTaskList} = useSystemIntents(docUrl);

  const handleToggle = (id: string) => {
    toggleTaskCompletion(id as TaskID);
  };

  const handleCreate = (text: string) => {
    createTask(text);
  };

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

        <PriorityTaskList onToggle={handleToggle} tasks={tasks} />
      </Stack>
    </Container>
  );
}
