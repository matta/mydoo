import {selectTaskEntities, type TaskID} from '@mydoo/tasklens';
import {useCallback} from 'react';
import {useSelector} from 'react-redux';
import {MovePickerModal} from '../../components/modals/move-picker-modal';
import {useTaskIntents} from '../intents/use-task-intents';
import {useValidParentTargets} from '../projections/use-valid-parent-targets';
import {useNavigationState} from '../ui/use-navigation-state';

export function MovePickerContainer() {
  const {modal, closeModal} = useNavigationState();

  const isOpen = modal?.type === 'move';
  const taskId = modal?.type === 'move' ? modal.taskId : undefined;

  const {moveTask} = useTaskIntents();
  const {roots, isLoading} = useValidParentTargets(taskId);
  const tasks = useSelector(selectTaskEntities);

  const task = taskId ? tasks[taskId] : undefined;

  const handleSelect = useCallback(
    (newParentId: TaskID | undefined) => {
      if (!taskId) return;

      moveTask(taskId, newParentId, undefined);
      closeModal();
    },
    [taskId, moveTask, closeModal],
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
