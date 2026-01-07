import { test } from './fixtures';

test.describe('Priority List', () => {
  test('seed data loads and renders tasks', async ({ plan }) => {
    // 1. Visit the app with ?seed=true to populate data
    await plan.primeWithSampleData();

    // 2. Wait for the list container to appear
    await plan.switchToDoView();

    // 3. Verify leaf tasks are rendered
    // The Do View shows only leaf tasks (Pass 7 hides containers with visible children).
    // Seed data includes:
    // - Research Requirements, Design UI Mocks (children of Project Alpha)
    // - Milk, Eggs, Bread (children of Buy Groceries)
    // - Unit Test (deepest leaf)
    // - Quick Task (standalone)
    await plan.verifyTaskVisible('Research Requirements');
    await plan.verifyTaskVisible('Milk');
    await plan.verifyTaskVisible('Quick Task');
  });
});
