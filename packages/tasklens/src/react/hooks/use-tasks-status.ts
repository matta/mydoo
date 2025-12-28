import {useSelector} from 'react-redux';
import {selectIsReady} from '../../store/slices/tasks-slice';

/**
 * Hook to check if the TaskLens store has received its initial data.
 */
export function useTasksStatus() {
  const isReady = useSelector(selectIsReady);
  return {isReady};
}
