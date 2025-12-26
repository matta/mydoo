import {useMediaQuery} from '@mantine/hooks';
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
  const {
    modal,
    closeModal,
    openCreateModal,
    openMoveModal,
    viewPath,
    popView,
    pushView,
    expandAll,
    setViewPath,
    setLastCreatedTaskId,
    setActiveTab,
  } = useNavigationState();

  const editingTaskId = modal?.type === 'edit' ? modal.taskId : undefined;

  const {task, parentTitle, descendantCount} = useTaskDetails(
    docUrl,
    editingTaskId ?? ('' as TaskID),
  );
  const {updateTask, createTask, deleteTask, indentTask, outdentTask} =
    useTaskIntents(docUrl);

  const isDesktop = useMediaQuery('(min-width: 768px)');

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

  /**
   * Calculate whether the task can be indented.
   * A task can only be indented if it has a previous sibling to become a child of.
   * This duplicates logic from useTaskIntents.indentTask() for UI state purposes.
   */
  let canIndent = false;
  if (task && doc) {
    const siblings = task.parentId
      ? doc.tasks[task.parentId]?.childTaskIds
      : doc.rootTaskIds;

    if (siblings && siblings.length > 0) {
      // Only enable if we have siblings and this task is not first
      const idx = siblings.indexOf(task.id);
      canIndent = idx > 0;
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

    let newTaskId: TaskID;

    if (modal.afterTaskId) {
      newTaskId = createTask(title, modal.parentId, {
        position: 'after',
        afterTaskId: modal.afterTaskId,
      });
    } else if (modal.position) {
      newTaskId = createTask(title, modal.parentId, {
        position: modal.position,
      });
    } else {
      newTaskId = createTask(title, modal.parentId);
    }

    // UX: Highlight & Reveal
    setLastCreatedTaskId(newTaskId);

    if (modal.parentId) {
      if (isDesktop) {
        // Desktop: Auto-expand the parent so the child is visible
        expandAll([modal.parentId]);
      } else {
        // Mobile: Auto-drill into the parent so the child is visible
        pushView(modal.parentId);
      }
    }
  };

  /**
   * Indents the task to become a child of its previous sibling.
   * @param taskId - The ID of the task to indent.
   */
  const handleIndent = (taskId: TaskID) => {
    indentTask(taskId);
  };

  /**
   * Outdents the task to become a sibling of its parent.
   *
   * **Mobile Edge Case**: If the user is zoomed into the parent context,
   * outdenting moves the task out of view. We auto-navigate up one level
   * before executing the move to keep the task visible after the operation.
   *
   * **Timing Note**: We check `task.parentId` from current React state (before
   * the Automerge operation), then navigate synchronously. The outdent operation
   * applies asynchronously, so the task data is stable during our check.
   *
   * @param taskId - The ID of the task to outdent.
   */
  const handleOutdent = (taskId: TaskID) => {
    if (task) {
      const currentHead = viewPath[viewPath.length - 1];
      if (currentHead && currentHead === task.parentId) {
        // Navigate up before the move completes to keep task visible
        popView();
      }
    }
    outdentTask(taskId);
  };

  const handleMove = (taskId: TaskID) => {
    // We are currently in the edit modal. We want to switch to the move modal.
    // openMoveModal(taskId) will update the 'modal' state to { type: 'move' ... }
    // This will cause TaskEditorContainer to render null (because isOpen check fails)
    // and MovePickerContainer to render (because its isOpen check passes).
    // Perfect.
    if (taskId) {
      // Import this from hook if it's not exposed?
      // useNavigationState exposes openMoveModal.
      // But I need to destructure it first.
      openMoveModal(taskId);
    }
  };

  /**
   * "Find in Plan" handler.
   * Closes the modal, expands ancestors (to ensure visibility),
   * and highlights the task (scroll + flash).
   */
  const handleFindInPlan = (taskId: TaskID) => {
    if (!doc) return;

    // 1. Calculate ancestry path
    const ancestors: TaskID[] = [];
    let currentId = doc.tasks[taskId]?.parentId;
    while (currentId && doc.tasks[currentId]) {
      ancestors.unshift(currentId);
      currentId = doc.tasks[currentId]?.parentId;
    }

    // 2. Ensure visibility based on viewport mode
    if (isDesktop) {
      // Desktop: Expand all ancestors so the tree opens up
      expandAll(ancestors);
      // Reset view path (if we were drilled down elsewhere) so we can see the full tree
      // (Optional: only do this if the ancestors aren't visible? For robustness, reset to root)
      setViewPath([]);
    } else {
      // Mobile: drill down to the parent
      // This sets the view path to [grandparent, parent]
      // So the list view shows the siblings of the target task.
      setViewPath(ancestors);
    }

    // 3. Highlight and Scroll
    setLastCreatedTaskId(taskId); // Re-use the "New Task" highlight mechanism

    // 4. Switch to Plan View
    setActiveTab('plan');

    // 5. Close Modal
    closeModal();
  };

  return (
    <TaskEditorModal
      opened={!!modal && (modal.type === 'create' || !!task)}
      onClose={handleClose}
      task={modal?.type === 'create' ? null : task}
      mode={
        modal?.type === 'create' || modal?.type === 'edit'
          ? modal.type
          : undefined
      }
      parentTitle={resolvedParentTitle}
      descendantCount={descendantCount}
      onSave={handleSave}
      onCreate={handleCreate}
      onAddSibling={handleAddSibling}
      onAddChild={handleAddChild}
      onDelete={handleDelete}
      onIndent={handleIndent}
      onOutdent={handleOutdent}
      onMove={handleMove}
      onFindInPlan={handleFindInPlan}
      canIndent={canIndent}
    />
  );
}
