import type {TaskID} from '@mydoo/tasklens';
import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useMemo,
  useState,
} from 'react';

/**
 * State manager for hierarchical tree navigation and interaction.
 *
 * Handles:
 * 1. **Expansion State**: Which parent nodes are expanded in the tree view.
 * 2. **Drill-down Navigation**: A stack of TaskIDs representing the user's
 *    current traversal path (used typically in mobile views or focused editing).
 */
export interface NavigationState {
  /** The currently active tab in the application shell. */
  activeTab: 'do' | 'plan';

  /** Set the active tab. */
  setActiveTab: (tab: 'do' | 'plan') => void;

  /** Collapse all currently expanded nodes. */
  collapseAll: () => void;

  /**
   * The ID of the task currently being viewed in drill-down mode.
   * Undefined means the user is at the root level.
   */
  currentViewId: TaskID | undefined;

  /**
   * Expand multiple nodes at once.
   * Useful for "Expand All" actions or restoring state.
   */
  expandAll: (ids: TaskID[]) => void;

  /** The set of currently expanded task IDs. */
  expandedIds: Set<TaskID>;

  /** Check if a specific task is currently expanded. */
  isExpanded: (id: TaskID) => boolean;

  /** Navigate up one level in the drill-down stack. */
  popView: () => void;

  /** Navigate down into a specific task (drill-down). */
  pushView: (id: TaskID) => void;

  /** Reset the drill-down stack to the root. */
  resetView: () => void;

  /**
   * Set the entire drill-down stack to a specific path.
   * Useful for jumping to a specific depth (e.g., breadcrumbs).
   */
  setViewPath: (ids: TaskID[]) => void;

  /** Toggle the expansion state of a specific task. */
  toggleExpanded: (id: TaskID) => void;

  /** The full history stack of drill-down navigation. */
  viewPath: TaskID[];
}

const NavigationContext = createContext<NavigationState | null>(null);

/**
 * Provider component that holds the Navigation State.
 */
export function NavigationProvider({children}: {children: ReactNode}) {
  // Active tab state
  const [activeTab, setActiveTab] = useState<'do' | 'plan'>('do');

  // Set of ID strings for expanded nodes
  const [expandedIds, setExpandedIds] = useState<Set<TaskID>>(new Set());

  // Stack of IDs for drill-down navigation (empty = root)
  const [viewPath, setViewPathState] = useState<TaskID[]>([]);

  const isExpanded = useCallback(
    (id: TaskID) => expandedIds.has(id),
    [expandedIds],
  );

  const toggleExpanded = useCallback((id: TaskID) => {
    setExpandedIds(prev => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  const expandAll = useCallback((ids: TaskID[]) => {
    setExpandedIds(prev => {
      const next = new Set(prev);
      for (const id of ids) {
        next.add(id);
      }
      return next;
    });
  }, []);

  const collapseAll = useCallback(() => {
    setExpandedIds(new Set());
  }, []);

  const pushView = useCallback((id: TaskID) => {
    setViewPathState(prev => {
      // Copy the array to mutate it for push
      const next = [...prev];
      next.push(id);
      return next;
    });
  }, []);

  const popView = useCallback(() => {
    setViewPathState(prev => {
      // Copy the array to mutate it for pop
      const next = [...prev];
      next.pop();
      return next;
    });
  }, []);

  const resetView = useCallback(() => {
    setViewPathState([]);
  }, []);

  const setViewPath = useCallback((ids: TaskID[]) => {
    setViewPathState(ids);
  }, []);

  const value = useMemo(
    () => ({
      activeTab,
      setActiveTab,
      collapseAll,
      currentViewId: viewPath.at(-1),
      expandAll,
      expandedIds,
      isExpanded,
      popView,
      pushView,
      resetView,
      setViewPath,
      toggleExpanded,
      viewPath,
    }),
    [
      activeTab,
      expandAll,
      expandedIds,
      isExpanded,
      popView,
      pushView,
      resetView,
      setViewPath,
      toggleExpanded,
      viewPath,
      collapseAll,
    ],
  );

  return (
    <NavigationContext.Provider value={value}>
      {children}
    </NavigationContext.Provider>
  );
}

/**
 * Hook to consume the Navigation State from Context.
 */
export function useNavigationState(): NavigationState {
  const context = useContext(NavigationContext);
  if (!context) {
    throw new Error(
      'useNavigationState must be used within a NavigationProvider',
    );
  }
  return context;
}
