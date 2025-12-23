import {Container, LoadingOverlay, Stack, Title} from '@mantine/core';
import type {DocumentHandle, TaskID} from '@mydoo/tasklens';

import {PriorityTaskList} from '../../components/composites/PriorityTaskList';
import {QuickAddInput} from '../../components/primitives/QuickAddInput';
import {useTaskIntents} from '../intents/useTaskIntents';
import {usePriorityList} from '../projections/usePriorityList';

export interface DoViewContainerProps {
  docUrl: DocumentHandle;
}

export function DoViewContainer({docUrl}: DoViewContainerProps) {
  const {tasks, isLoading} = usePriorityList(docUrl);
  const {createTask, toggleTaskCompletion} = useTaskIntents(docUrl);

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
        <Title order={2}>Priorities</Title>

        <QuickAddInput onAdd={handleCreate} />

        <PriorityTaskList onToggle={handleToggle} tasks={tasks} />
      </Stack>
    </Container>
  );
}
