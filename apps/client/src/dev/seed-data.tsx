import { useDocHandle } from "@automerge/automerge-repo-react-hooks";
import {
  type TunnelState,
  useTaskActions,
  useTaskLensDocUrl,
} from "@mydoo/tasklens";
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
  const alphaId = actions.createTask("Project Alpha", undefined, undefined, {
    importance: IMPORTANCE_HIGH,
  });

  actions.createTask("Research Requirements", alphaId, undefined, {
    importance: IMPORTANCE_HIGH,
  });
  actions.createTask("Design UI Mocks", alphaId, undefined, {
    importance: IMPORTANCE_HIGH,
  });

  // 2. Buy Groceries
  const groceriesId = actions.createTask(
    "Buy Groceries",
    undefined,
    undefined,
    {
      importance: IMPORTANCE_NORMAL,
    },
  );

  actions.createTask("Milk", groceriesId, undefined, {
    importance: IMPORTANCE_NORMAL,
  });
  actions.createTask("Eggs", groceriesId, undefined, {
    importance: IMPORTANCE_NORMAL,
  });
  actions.createTask("Bread", groceriesId, undefined, {
    importance: IMPORTANCE_NORMAL,
  });

  // 3. Deep Work Project
  const deepId = actions.createTask("Deep Work Project", undefined, undefined, {
    importance: IMPORTANCE_MEDIUM,
  });

  const moduleId = actions.createTask("Module A", deepId, undefined, {
    importance: IMPORTANCE_MEDIUM,
  });

  const componentId = actions.createTask("Component X", moduleId, undefined, {
    importance: IMPORTANCE_MEDIUM,
  });

  actions.createTask("Unit Test", componentId, undefined, {
    importance: IMPORTANCE_MEDIUM,
  });

  // 4. Standalone Task
  actions.createTask("Quick Task", undefined, undefined, {
    importance: IMPORTANCE_LOW,
  });
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
