import {test} from './fixtures';

test.describe('Task Creation with Defaults', () => {
  test('should create task with default placeId via Quick Add', async ({
    plan,
    page,
  }) => {
    await plan.primeWithSampleData();

    // The Quick Add input is only available in the "Do" (Priorities) view
    await plan.switchToDoView();

    // Create a task via Quick Add
    const input = page.getByPlaceholder('Add a new task...');
    await input.fill('Test Task with Defaults');
    await input.press('Enter');

    // Verify task appears in the list (Priority list)
    await plan.verifyTaskVisible('Test Task with Defaults');
  });

  test('should create task and display in priority list', async ({
    plan,
    page,
  }) => {
    await plan.primeWithSampleData();
    await plan.switchToDoView();

    // Create multiple tasks
    const input = page.getByPlaceholder('Add a new task...');

    await input.fill('First Task');
    await input.press('Enter');

    await input.fill('Second Task');
    await input.press('Enter');

    // Both tasks should be visible in the priority list
    await plan.verifyTaskVisible('First Task');
    await plan.verifyTaskVisible('Second Task');
  });

  test('should persist task after page reload', async ({plan, page}) => {
    await plan.primeWithSampleData();
    await plan.switchToDoView();

    // Create a task
    const input = page.getByPlaceholder('Add a new task...');
    await input.fill('Persistent Task');
    await input.press('Enter');

    await plan.verifyTaskVisible('Persistent Task');

    // Wait for persistence to flush
    await page.waitForTimeout(1000);

    // Reload the page
    await page.reload();
    await plan.switchToDoView();

    // Task should still be visible
    await plan.verifyTaskVisible('Persistent Task');
  });
});
