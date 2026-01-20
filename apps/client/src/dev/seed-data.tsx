import type { AutomergeUrl } from "@automerge/automerge-repo";
import { useDocHandle } from "@automerge/automerge-repo-react-hooks";
import {
  selectStoreReady,
  TaskActions,
  type TaskCreateInput,
  type TaskID,
} from "@mydoo/tasklens";
import type { TunnelState } from "@mydoo/tasklens/persistence";
import { getTaskCount } from "@mydoo/tasklens/test";
import { useEffect, useRef } from "react";
import { type AppDispatch, useAppDispatch, useAppSelector } from "../store";

// Importance constants
const IMPORTANCE_HIGH = 1.0;
const IMPORTANCE_MEDIUM = 0.8;
const IMPORTANCE_NORMAL = 0.5;
const IMPORTANCE_LOW = 0.1;

export function seedHierarchicalData(dispatch: AppDispatch) {
  const create = (props: TaskCreateInput & { parentId?: TaskID }) => {
    const id = crypto.randomUUID() as TaskID;
    dispatch(TaskActions.createTask({ ...props, id }));
    return id;
  };

  // 1. Project Alpha
  const alphaId = create({
    title: "Project Alpha",
    importance: IMPORTANCE_HIGH,
  });

  create({
    title: "Research Requirements",
    parentId: alphaId,
    importance: IMPORTANCE_HIGH,
  });
  create({
    title: "Design UI Mocks",
    parentId: alphaId,
    importance: IMPORTANCE_HIGH,
  });

  // 2. Buy Groceries
  const groceriesId = create({
    title: "Buy Groceries",
    importance: IMPORTANCE_NORMAL,
  });

  create({
    title: "Milk",
    parentId: groceriesId,
    importance: IMPORTANCE_NORMAL,
  });
  create({
    title: "Eggs",
    parentId: groceriesId,
    importance: IMPORTANCE_NORMAL,
  });
  create({
    title: "Bread",
    parentId: groceriesId,
    importance: IMPORTANCE_NORMAL,
  });

  // 3. Deep Work Project
  const deepId = create({
    title: "Deep Work Project",
    importance: IMPORTANCE_MEDIUM,
  });

  const moduleId = create({
    title: "Module A",
    parentId: deepId,
    importance: IMPORTANCE_MEDIUM,
  });

  const componentId = create({
    title: "Component X",
    parentId: moduleId,
    importance: IMPORTANCE_MEDIUM,
  });

  create({
    title: "Unit Test",
    parentId: componentId,
    importance: IMPORTANCE_MEDIUM,
  });

  // 4. Standalone Task
  create({ title: "Quick Task", importance: IMPORTANCE_LOW });
}

/**
 * Helper component that seeds data when the `?seed=true` query param is present.
 */
export function SeedData({ docUrl }: { docUrl: AutomergeUrl }) {
  const dispatch = useAppDispatch();
  const handle = useDocHandle<TunnelState>(docUrl);
  const seeded = useRef(false);
  const isReady = useAppSelector(selectStoreReady);

  useEffect(() => {
    if (!handle || !isReady) return;

    // We need the doc to check if it's empty
    const doc = handle.doc();
    const params = new URLSearchParams(window.location.search);
    if (params.get("seed") === "true" && doc && !seeded.current) {
      const taskCount = getTaskCount(doc);
      if (taskCount === 0) {
        seeded.current = true;
        seedHierarchicalData(dispatch);
      }
    }
  }, [handle, dispatch, isReady]);

  return null;
}
