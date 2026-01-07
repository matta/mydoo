import { useMediaQuery } from '@mantine/hooks';
import {
  selectRootTaskIds,
  selectTaskEntities,
  type Task,
  type TaskID,
} from '@mydoo/tasklens';
import { useCallback, useEffect } from 'react';
import { useSelector } from 'react-redux';

import { TaskEditorModal } from '../../components/modals/task-editor-modal';
import { useTaskIntents } from '../intents/use-task-intents';
import { useTaskDetails } from '../projections/use-task-details';
import { useNavigationState } from '../ui/use-navigation-state';

/**
 * Container that connects the TaskEditorModal to the application state.
 *
 * Responsibilities:
 * - Monitors `editingTaskId` from NavigationState to determine visibility.
 * - Fetches task data using `useTaskDetails`.
 * - Provides action handlers (save, add sibling, add child, delete) via `useTaskIntents`.
 */
export function TaskEditorContainer() {
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

  const { task, parentTitle, descendantCount, isLoading } =
    useTaskDetails(editingTaskId);
  const { updateTask, createTask, deleteTask, indentTask, outdentTask } =
    useTaskIntents();

  // Auto-close if task is missing (deleted remotely)
  useEffect(() => {
    if (editingTaskId && !task && !isLoading && modal?.type === 'edit') {
      closeModal();
    }
  }, [editingTaskId, task, isLoading, modal?.type, closeModal]);

  const isDesktop = useMediaQuery('(min-width: 768px)');

  // Resolve parent title for Create Mode
  const tasks = useSelector(selectTaskEntities);
  const rootTaskIds = useSelector(selectRootTaskIds);

  let resolvedParentTitle = parentTitle;

  // We need to pass a ComputedTask to the modal, but our internal state in Editor
  // is usually partial or persisted.
  // If we have a full task object from the doc, project it.
  // If we are creating, we pass null.
  // But if we have partial updates, we might need to handle that.
  // TaskEditorModal expects `Task | undefined` (where Task is ComputedTask alias).
  const editorState = useCallback(() => {
    if (modal?.type === 'create') {
      if (!modal.parentId) return { task: undefined, parent: undefined };
      return { task: undefined, parent: tasks[modal.parentId] };
    }

    // Edit mode
    const editingTask = task as Task | undefined;
    if (!editingTask) return { task: undefined, parent: undefined };

    const parentId = editingTask.parentId;
    return {
      task: editingTask,
      parent: parentId ? (tasks[parentId] as Task) : undefined,
    };
  }, [tasks, modal, task]);

  if (modal?.type === 'create') {
    const parentId = modal.parentId;
    if (parentId && tasks[parentId]) {
      resolvedParentTitle = tasks[parentId].title;
    } else {
      resolvedParentTitle = ''; // Root or unknown
    }
  }

  /**
   * Calculate whether the task can be indented.
   * A task can only be indented if it has a previous sibling to become a child of.
   * This duplicates logic from useTaskIntents.indentTask() for UI state purposes.
   */
  let canIndent = false;
  if (task) {
    const siblings = task.parentId
      ? tasks[task.parentId]?.childTaskIds
      : rootTaskIds;

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
   * @param props - Additional task properties (importance, effort, etc.)
   */
  const handleCreate = (title: string, props?: Partial<Task>) => {
    if (modal?.type !== 'create') return;

    let newTaskId: TaskID;

    if (modal.afterTaskId) {
      newTaskId = createTask(
        title,
        modal.parentId,
        {
          position: 'after',
          afterTaskId: modal.afterTaskId,
        },
        props,
      );
    } else if (modal.position) {
      newTaskId = createTask(
        title,
        modal.parentId,
        {
          position: modal.position,
        },
        props,
      );
    } else {
      newTaskId = createTask(title, modal.parentId, undefined, props);
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
    if (taskId) {
      openMoveModal(taskId);
    }
  };

  /**
   * "Find in Plan" handler.
   * Closes the modal, expands ancestors (to ensure visibility),
   * and highlights the task (scroll + flash).
   */
  const handleFindInPlan = (taskId: TaskID) => {
    // 1. Calculate ancestry path
    const ancestors: TaskID[] = [];
    let currentId = tasks[taskId]?.parentId;
    while (currentId && tasks[currentId]) {
      ancestors.unshift(currentId);
      currentId = tasks[currentId]?.parentId;
    }

    // 2. Ensure visibility based on viewport mode
    if (isDesktop) {
      // Desktop: Expand all ancestors so the tree opens up
      expandAll(ancestors);
      // Reset view path (if we were drilled down elsewhere) so we can see the full tree
      setViewPath([]);
    } else {
      // Mobile: drill down to the parent
      setViewPath(ancestors);
    }

    // 3. Highlight and Scroll
    setLastCreatedTaskId(taskId); // Re-use the "New Task" highlight mechanism

    // 4. Switch to Plan View
    setActiveTab('plan');

    // 5. Close Modal
    closeModal();
  };

  const currentEditorState = editorState();

  return (
    <TaskEditorModal
      opened={!!modal && (modal.type === 'create' || !!task)}
      onClose={handleClose}
      task={currentEditorState.task} // Now correctly typed as computed task or undefined
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
