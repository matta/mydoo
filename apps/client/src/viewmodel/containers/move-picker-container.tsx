import type {TaskID} from '@mydoo/tasklens';
import {useTunnel} from '@mydoo/tasklens';
import {useCallback} from 'react';
import {MovePickerModal} from '../../components/modals/move-picker-modal';
import {useTaskIntents} from '../intents/use-task-intents';
import {useValidParentTargets} from '../projections/use-valid-parent-targets';
import {useNavigationState} from '../ui/use-navigation-state';
import {useDocument} from '../use-document';

export function MovePickerContainer() {
  const {modal, closeModal} = useNavigationState();
  const docUrl = useDocument();

  const isOpen = modal?.type === 'move';
  const taskId = modal?.type === 'move' ? modal.taskId : undefined;

  const {moveTask} = useTaskIntents(docUrl);
  const {roots, isLoading} = useValidParentTargets(docUrl, taskId);
  const {doc} = useTunnel(docUrl);

  const task = taskId && doc?.tasks ? doc.tasks[taskId] : undefined;

  const handleSelect = useCallback(
    (newParentId: TaskID | undefined) => {
      if (!docUrl || !taskId) return;

      moveTask(taskId, newParentId, undefined);
      closeModal();
    },
    [docUrl, taskId, moveTask, closeModal],
  );

  if (!isOpen || !taskId) return null;

  return (
    <MovePickerModal
      opened={isOpen}
      onClose={closeModal}
      roots={isLoading ? [] : roots}
      onSelect={handleSelect}
      taskTitle={task?.title ?? 'Task'}
    />
  );
}
