import { selectBalanceData } from '@mydoo/tasklens';
import { useSelector } from 'react-redux';

/**
 * useBalanceData Hook
 *
 * Provides the balance allocation data for the Balance View.
 * Uses the Redux selector for global memoization.
 */
export function useBalanceData() {
  return useSelector(selectBalanceData);
}
