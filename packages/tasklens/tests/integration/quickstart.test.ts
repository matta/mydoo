import {describe, expect, it} from 'vitest';

import {getPrioritizedTasks} from '../../src/domain/priority';
import {TunnelStore} from '../../src/persistence/store';

describe('Quickstart Integration', () => {
  it('should run the quickstart scenario', () => {
    // 1. Initialize Store
    const store = new TunnelStore();

    // 2. Create Data
    const rootGoal = store.createTask({
      title: 'Work',
      desiredCredits: 100,
    });

    const task = store.createTask({
      title: 'Email',
      parentId: rootGoal.id,
      creditIncrement: 1.0,
    });

    // 3. Get Todo List (Calculates scores on demand)
    const todos = getPrioritizedTasks(store.doc, {});

    // Email should be visible and sorted
    expect(todos.length).toBeGreaterThan(0);
    const firstTodo = todos[0];
    if (!firstTodo) throw new Error('Expected at least one todo');
    expect(firstTodo.id).toBe(task.id);
    // Explicit cast to access hidden/optional priority for integration verification
    // biome-ignore lint/suspicious/noExplicitAny: integration test verifying hidden field
    expect((firstTodo as any).priority ?? 0).toBeGreaterThan(0);

    // Verify Work (Root) is hidden (Container Visibility Pass 7)
    // Work has a visible child (Email), so Work should be hidden from the todo list.
    const workInTodos = todos.find((t) => t.id === rootGoal.id);
    expect(workInTodos).toBeUndefined();
  });
});
