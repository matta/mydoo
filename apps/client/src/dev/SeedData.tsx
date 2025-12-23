import type {DocumentHandle, TaskID} from '@mydoo/tasklens';
import {type TunnelHookResult, useTunnel} from '@mydoo/tasklens';
import {useEffect, useRef} from 'react';

// Importance constants for seed data
// Importance is a user-assigned value (0.0 to 1.0) that influences the priority algorithm.
// Priority itself is computed by the algorithm, not set directly.
const IMPORTANCE_HIGH = 1.0; // Most important tasks
const IMPORTANCE_MEDIUM = 0.8; // Upper-middle importance
const IMPORTANCE_NORMAL = 0.5; // Standard importance
const IMPORTANCE_LOW = 0.1; // Least important tasks

/**
 * Seeds the Automerge document with a rich set of hierarchical task data.
 *
 * This function is used to populate the application with sample data for development,
 * testing, and demonstration purposes. It creates a mix of:
 * - Nested project structures (up to 4 levels deep) to test tree visualization.
 * - Flat lists to test basic rendering.
 * - Varied importance levels to test the priority algorithm's sorting behavior.
 *
 * Note: Only `importance` is set here, not `priority`. Priority is a computed output
 * of the prioritization algorithm (Pass 6) and should not be set directly.
 *
 * @param ops - The operations object from `useTunnel` to perform mutations.
 */
export function seedHierarchicalData(ops: TunnelHookResult['ops']) {
  // 1. Project Alpha (Parent)
  // Represents a standard project with direct children.
  const alphaId = crypto.randomUUID() as TaskID;
  ops.add({
    id: alphaId,
    title: 'Project Alpha',
    importance: IMPORTANCE_HIGH,
  });

  // Children of Alpha
  ops.add({
    title: 'Research Requirements',
    parentId: alphaId,
    importance: IMPORTANCE_HIGH,
  });
  ops.add({
    title: 'Design UI Mocks',
    parentId: alphaId,
    importance: IMPORTANCE_HIGH,
  });

  // 2. Buy Groceries (Parent)
  // Represents a personal task list with multiple items.
  const groceriesId = crypto.randomUUID() as TaskID;
  ops.add({
    id: groceriesId,
    title: 'Buy Groceries',
    importance: IMPORTANCE_NORMAL,
  });

  // Children of Groceries
  ops.add({
    title: 'Milk',
    parentId: groceriesId,
    importance: IMPORTANCE_NORMAL,
  });
  ops.add({
    title: 'Eggs',
    parentId: groceriesId,
    importance: IMPORTANCE_NORMAL,
  });
  ops.add({
    title: 'Bread',
    parentId: groceriesId,
    importance: IMPORTANCE_NORMAL,
  });

  // 3. Deep Work Project (4 levels)
  // Critical for testing deep nesting support in the Plan View (indentation, chevrons).
  // Hierarchy: Project -> Module -> Component -> Task
  const deepId = crypto.randomUUID() as TaskID;
  ops.add({
    id: deepId,
    title: 'Deep Work Project',
    importance: IMPORTANCE_MEDIUM,
  });

  const moduleId = crypto.randomUUID() as TaskID;
  ops.add({
    id: moduleId,
    title: 'Module A',
    parentId: deepId,
    importance: IMPORTANCE_MEDIUM,
  });

  const componentId = crypto.randomUUID() as TaskID;
  ops.add({
    id: componentId,
    title: 'Component X',
    parentId: moduleId,
    importance: IMPORTANCE_MEDIUM,
  });

  ops.add({
    title: 'Unit Test',
    parentId: componentId,
    importance: IMPORTANCE_MEDIUM,
  });

  // 4. Standalone Task
  // A simple task with no children to verify leaf node rendering.
  ops.add({
    title: 'Quick Task',
    importance: IMPORTANCE_LOW,
  });
}

/**
 * Helper component that seeds data when the `?seed=true` query param is present.
 *
 * Use Case:
 * - Automated E2E tests use this to ensure consistent initial state.
 * - Developers can append `?seed=true` to the URL to quickly hydrate a fresh environment.
 *
 * Logic:
 * - Checks for `seed=true` in URL search params.
 * - Checks if the document is empty (has 0 tasks).
 * - Only runs once per mount (via `current` ref) to prevent duplicate seeding.
 *
 * Usage: <SeedData docUrl={docUrl} />
 */
export function SeedData({docUrl}: {docUrl: DocumentHandle}) {
  const {doc, ops} = useTunnel(docUrl);
  // Ref to track if we have already attempted to seed in this session
  const seeded = useRef(false);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);

    // Condition: ?seed=true present, doc loaded, and not yet seeded by this component
    if (params.get('seed') === 'true' && doc && !seeded.current) {
      const taskCount = Object.keys(doc.tasks).length;

      // Safety check: Only seed if the document is actually empty to avoid polluting existing data
      if (taskCount === 0) {
        seeded.current = true;
        seedHierarchicalData(ops);
      }
    }
  }, [doc, ops]);

  // Component renders nothing - exists only to trigger seeding side effect
  return null;
}
