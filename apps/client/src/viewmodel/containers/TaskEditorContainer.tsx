import type {DocumentHandle, Task, TaskID} from '@mydoo/tasklens';
import {TaskEditorModal} from '../../components/modals/TaskEditorModal';
import {useTaskIntents} from '../intents/useTaskIntents';
import {useTaskDetails} from '../projections/useTaskDetails';
import {useNavigationState} from '../ui/useNavigationState';

interface TaskEditorContainerProps {
  docUrl: DocumentHandle;
}

/**
 * Container that connects the TaskEditorModal to the application state.
 *
 * Responsibilities:
 * - Monitors `editingTaskId` from NavigationState to determine visibility.
 * - Fetches task data using `useTaskDetails`.
 * - Provides action handlers (save, add sibling, add child, delete) via `useTaskIntents`.
 */
export function TaskEditorContainer({docUrl}: TaskEditorContainerProps) {
  const {editingTaskId, setEditingTaskId} = useNavigationState();
  const {task, parentTitle, descendantCount} = useTaskDetails(
    docUrl,
    editingTaskId ?? ('' as TaskID),
  );
  const {updateTask, createTask, deleteTask} = useTaskIntents(docUrl);

  /** Closes the Task Editor modal by clearing the editing state. */
  const handleClose = () => setEditingTaskId(null);

  /**
   * Persists task changes to the document.
   * @param taskId - The ID of the task being updated.
   * @param updates - Partial task object with fields to update.
   */
  const handleSave = (taskId: TaskID, updates: Partial<Task>) => {
    updateTask(taskId, updates);
  };

  /**
   * Creates a new sibling task under the same parent.
   * @param parentId - The parent task ID, or undefined for root-level tasks.
   */
  const handleAddSibling = (parentId: TaskID | undefined) => {
    createTask('New Task', parentId);
  };

  /**
   * Creates a new child task under the specified parent.
   * @param parentId - The parent task ID.
   */
  const handleAddChild = (parentId: TaskID) => {
    createTask('New Subtask', parentId);
  };

  /**
   * Deletes a task, with confirmation for tasks that have children.
   * TODO: Replace browser confirm() with Mantine modal (deferred polish).
   * @param taskId - The ID of the task to delete.
   * @param hasChildren - Whether the task has child tasks.
   */
  const handleDelete = (taskId: TaskID, hasChildren: boolean) => {
    if (hasChildren) {
      if (
        confirm(
          `This task has ${descendantCount} descendants. Are you sure you want to delete it and all its children?`,
        )
      ) {
        deleteTask(taskId);
        handleClose();
      }
    } else {
      deleteTask(taskId);
      handleClose();
    }
  };

  return (
    <TaskEditorModal
      opened={!!editingTaskId && !!task}
      onClose={handleClose}
      task={task}
      parentTitle={parentTitle}
      descendantCount={descendantCount}
      onSave={handleSave}
      onAddSibling={handleAddSibling}
      onAddChild={handleAddChild}
      onDelete={handleDelete}
    />
  );
}
