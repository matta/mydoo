import { useDocHandle } from "@automerge/automerge-repo-react-hooks";
import { useTaskActions, useTaskLensDocUrl } from "@mydoo/tasklens";
import type { TunnelState } from "@mydoo/tasklens/persistence";
import { useEffect, useRef } from "react";

// Importance constants
const IMPORTANCE_HIGH = 1.0;
const IMPORTANCE_MEDIUM = 0.8;
const IMPORTANCE_NORMAL = 0.5;
const IMPORTANCE_LOW = 0.1;

export function seedHierarchicalData(
  actions: ReturnType<typeof useTaskActions>,
) {
  // 1. Project Alpha
  const alphaId = actions.createTask({
    title: "Project Alpha",
    importance: IMPORTANCE_HIGH,
  });

  actions.createTask({
    title: "Research Requirements",
    parentId: alphaId,
    importance: IMPORTANCE_HIGH,
  });
  actions.createTask({
    title: "Design UI Mocks",
    parentId: alphaId,
    importance: IMPORTANCE_HIGH,
  });

  // 2. Buy Groceries
  const groceriesId = actions.createTask({
    title: "Buy Groceries",
    importance: IMPORTANCE_NORMAL,
  });

  actions.createTask({
    title: "Milk",
    parentId: groceriesId,
    importance: IMPORTANCE_NORMAL,
  });
  actions.createTask({
    title: "Eggs",
    parentId: groceriesId,
    importance: IMPORTANCE_NORMAL,
  });
  actions.createTask({
    title: "Bread",
    parentId: groceriesId,
    importance: IMPORTANCE_NORMAL,
  });

  // 3. Deep Work Project
  const deepId = actions.createTask({
    title: "Deep Work Project",
    importance: IMPORTANCE_MEDIUM,
  });

  const moduleId = actions.createTask({
    title: "Module A",
    parentId: deepId,
    importance: IMPORTANCE_MEDIUM,
  });

  const componentId = actions.createTask({
    title: "Component X",
    parentId: moduleId,
    importance: IMPORTANCE_MEDIUM,
  });

  actions.createTask({
    title: "Unit Test",
    parentId: componentId,
    importance: IMPORTANCE_MEDIUM,
  });

  // 4. Standalone Task
  actions.createTask({ title: "Quick Task", importance: IMPORTANCE_LOW });
}

/**
 * Helper component that seeds data when the `?seed=true` query param is present.
 */
export function SeedData() {
  const actions = useTaskActions();
  const docUrl = useTaskLensDocUrl();
  // biome-ignore lint/suspicious/noExplicitAny: internal type erasure
  const handle = useDocHandle(docUrl as any);
  const seeded = useRef(false);

  useEffect(() => {
    if (!handle) return;

    // We need the doc to check if it's empty
    const doc = handle.doc() as TunnelState | undefined;
    const params = new URLSearchParams(window.location.search);
    if (params.get("seed") === "true" && doc && !seeded.current) {
      const taskCount = Object.keys(doc.tasks || {}).length;
      if (taskCount === 0 && !seeded.current) {
        seeded.current = true;
        seedHierarchicalData(actions);
      }
    }
  }, [handle, actions]);

  return null;
}
