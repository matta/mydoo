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
 * State representing the content of the task editor modal.
 */
export type ModalState =
  | {type: 'edit'; taskId: TaskID}
  | {
      type: 'create';
      parentId: TaskID | undefined;
      afterTaskId: TaskID | undefined;
      position?: 'start' | 'end';
    };

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

  /**
   * The current state of the task editor modal.
   * Undefined means the modal is closed.
   */
  modal: ModalState | undefined;

  /** Opens the modal in Edit Mode for a specific task. */
  openEditModal: (id: TaskID) => void;

  /** Opens the modal in Create Mode. */
  openCreateModal: (
    parentId?: TaskID,
    afterTaskId?: TaskID,
    position?: 'start' | 'end',
  ) => void;

  /** Closes the task editor modal. */
  closeModal: () => void;

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

  // Unified modal state (undefined = closed)
  const [modal, setModal] = useState<ModalState | undefined>(undefined);

  const openEditModal = useCallback((taskId: TaskID) => {
    setModal({type: 'edit', taskId});
  }, []);

  const openCreateModal = useCallback(
    (parentId?: TaskID, afterTaskId?: TaskID, position?: 'start' | 'end') => {
      setModal({
        type: 'create',
        parentId,
        afterTaskId,
        ...(position ? {position} : {}),
      });
    },
    [],
  );

  const closeModal = useCallback(() => {
    setModal(undefined);
  }, []);

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
      modal,
      openEditModal,
      openCreateModal,
      closeModal,
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
      modal,
      openEditModal,
      openCreateModal,
      closeModal,
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
