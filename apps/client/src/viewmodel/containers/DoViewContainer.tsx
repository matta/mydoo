import {Container, LoadingOverlay, Stack, Title} from '@mantine/core';
import type {DocumentHandle} from '@mydoo/tasklens';

import {PriorityTaskList} from '../../components/composites/PriorityTaskList';
import {usePriorityList} from '../projections/usePriorityList';

export interface DoViewContainerProps {
  docUrl: DocumentHandle;
}

export function DoViewContainer({docUrl}: DoViewContainerProps) {
  const {tasks, isLoading} = usePriorityList(docUrl);

  const handleComplete = (id: string) => {
    // TODO: Implement completion logic in next phase (useTaskIntents)
    console.log('Complete task:', id);
  };

  return (
    <Container pos="relative" py="xl" size="sm">
      <LoadingOverlay visible={isLoading} />

      <Stack gap="xl">
        <Title order={2}>Priorities</Title>

        <PriorityTaskList onComplete={handleComplete} tasks={tasks} />
      </Stack>
    </Container>
  );
}
