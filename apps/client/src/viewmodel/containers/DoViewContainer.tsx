import {
  Button,
  Container,
  Group,
  LoadingOverlay,
  Stack,
  Title,
} from '@mantine/core';
import type {DocumentHandle, Task, TaskID} from '@mydoo/tasklens';
import {IconRefresh} from '@tabler/icons-react';
import {useCallback, useState} from 'react';

import {PriorityTaskList} from '../../components/composites/PriorityTaskList';
import {DeleteConfirmModal} from '../../components/modals/DeleteConfirmModal';
import {TaskEditorModal} from '../../components/modals/TaskEditorModal';
import {QuickAddInput} from '../../components/primitives/QuickAddInput';
import {useSystemIntents} from '../intents/useSystemIntents';
import {useTaskIntents} from '../intents/useTaskIntents';
import {usePriorityList} from '../projections/usePriorityList';
import {useTaskDetails} from '../projections/useTaskDetails';

export interface DoViewContainerProps {
  docUrl: DocumentHandle;
}

/**
 * DoViewContainer: Main container for the "Do" view (priority list).
 *
 * Orchestrates:
 * - Task list display via usePriorityList
 * - Quick task creation
 * - Task editing via TaskEditorModal
 * - Task deletion with confirmation via DeleteConfirmModal
 */
export function DoViewContainer({docUrl}: DoViewContainerProps) {
  const {tasks, isLoading} = usePriorityList(docUrl);
  const {createTask, toggleTaskCompletion, deleteTask, updateTask} =
    useTaskIntents(docUrl);
  const {refreshTaskList} = useSystemIntents(docUrl);

  // Modal state
  const [selectedTaskId, setSelectedTaskId] = useState<TaskID | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [taskToDelete, setTaskToDelete] = useState<{
    id: TaskID;
    title: string;
    descendantCount: number;
  } | null>(null);

  // Get details for the selected task
  const {task, parentTitle, descendantCount} = useTaskDetails(
    docUrl,
    selectedTaskId ?? ('' as TaskID),
  );

  const handleToggle = useCallback(
    (id: TaskID) => {
      toggleTaskCompletion(id);
    },
    [toggleTaskCompletion],
  );

  const handleTitleTap = useCallback((id: TaskID) => {
    setSelectedTaskId(id);
  }, []);

  const handleCloseEditor = useCallback(() => {
    setSelectedTaskId(null);
  }, []);

  const handleCreate = useCallback(
    (text: string) => {
      createTask(text);
    },
    [createTask],
  );

  const handleSave = useCallback(
    (taskId: TaskID, updates: Partial<Task>) => {
      updateTask(taskId, updates);
    },
    [updateTask],
  );

  const handleAddSibling = useCallback(
    (parentId: TaskID | undefined) => {
      createTask('New Task', parentId);
    },
    [createTask],
  );

  const handleAddChild = useCallback(
    (parentId: TaskID) => {
      createTask('New Subtask', parentId);
    },
    [createTask],
  );

  const handleDelete = useCallback(
    (taskId: TaskID, hasChildren: boolean) => {
      const taskToDeleteData = tasks.find(t => t.id === taskId);
      if (!taskToDeleteData) return;

      if (hasChildren) {
        // Show confirmation modal
        setTaskToDelete({
          id: taskId,
          title: taskToDeleteData.title,
          descendantCount,
        });
        setShowDeleteConfirm(true);
      } else {
        // Delete directly without confirmation
        deleteTask(taskId);
        setSelectedTaskId(null);
      }
    },
    [tasks, descendantCount, deleteTask],
  );

  const handleConfirmDelete = useCallback(() => {
    if (taskToDelete) {
      deleteTask(taskToDelete.id);
      setTaskToDelete(null);
      setSelectedTaskId(null);
    }
  }, [taskToDelete, deleteTask]);

  const handleCloseDeleteConfirm = useCallback(() => {
    setShowDeleteConfirm(false);
    setTaskToDelete(null);
  }, []);

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

      {/* Task Editor Modal */}
      <TaskEditorModal
        descendantCount={descendantCount}
        onAddChild={handleAddChild}
        onAddSibling={handleAddSibling}
        onClose={handleCloseEditor}
        onDelete={handleDelete}
        onSave={handleSave}
        opened={selectedTaskId !== null}
        parentTitle={parentTitle}
        task={task}
      />

      {/* Delete Confirmation Modal */}
      <DeleteConfirmModal
        descendantCount={taskToDelete?.descendantCount ?? 0}
        onClose={handleCloseDeleteConfirm}
        onConfirm={handleConfirmDelete}
        opened={showDeleteConfirm}
        taskTitle={taskToDelete?.title ?? ''}
      />
    </Container>
  );
}
