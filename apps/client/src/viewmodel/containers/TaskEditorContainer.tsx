import {
  type DocumentHandle,
  type Task,
  type TaskID,
  useTunnel,
} from '@mydoo/tasklens';
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
  const {modal, closeModal, openCreateModal} = useNavigationState();
  const editingTaskId = modal?.type === 'edit' ? modal.taskId : undefined;

  const {task, parentTitle, descendantCount} = useTaskDetails(
    docUrl,
    editingTaskId ?? ('' as TaskID),
  );
  const {updateTask, createTask, deleteTask} = useTaskIntents(docUrl);

  // Resolve parent title for Create Mode
  const {doc} = useTunnel(docUrl);
  let resolvedParentTitle = parentTitle;

  if (modal?.type === 'create' && doc) {
    if (modal.parentId) {
      resolvedParentTitle = doc.tasks[modal.parentId]?.title ?? null;
    } else {
      resolvedParentTitle = null; // Root
    }
  }

  /** Closes the Task Editor modal by clearing the editing state. */
  const handleClose = () => closeModal();

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
    openCreateModal(parentId, editingTaskId);
  };

  /**
   * Creates a new child task under the specified parent.
   * @param parentId - The parent task ID.
   */
  const handleAddChild = (parentId: TaskID) => {
    openCreateModal(parentId);
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

  /**
   * Handles creation of a new task from the modal.
   * @param title - The title of the new task.
   */
  const handleCreate = (title: string) => {
    if (modal?.type !== 'create') return;

    if (modal.afterTaskId) {
      createTask(title, modal.parentId, {
        position: 'after',
        afterTaskId: modal.afterTaskId,
      });
    } else {
      createTask(title, modal.parentId);
    }
  };

  return (
    <TaskEditorModal
      opened={!!modal && (modal.type === 'create' || !!task)}
      onClose={handleClose}
      task={modal?.type === 'create' ? null : task}
      mode={modal?.type}
      parentTitle={resolvedParentTitle}
      descendantCount={descendantCount}
      onSave={handleSave}
      onCreate={handleCreate}
      onAddSibling={handleAddSibling}
      onAddChild={handleAddChild}
      onDelete={handleDelete}
    />
  );
}
