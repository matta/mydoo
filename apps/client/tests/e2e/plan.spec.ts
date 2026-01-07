import { test } from './fixtures';

test.describe('Plan View', () => {
  test.beforeEach(async ({ plan }) => {
    await plan.primeWithSampleData();
  });

  test('should render task hierarchy', async ({ plan }) => {
    await test.step('Verify initial collapsed state', async () => {
      await plan.switchToPlanView();
      await plan.verifyTaskVisible('Project Alpha');
      await plan.verifyTaskVisible('Buy Groceries');
      await plan.verifyTaskHidden('Research Requirements');
    });

    await test.step('Expand parent and verify children', async () => {
      await plan.toggleExpand('Project Alpha');
      await plan.verifyTaskVisible('Research Requirements');
      await plan.verifyTaskVisible('Design UI Mocks');
    });
  });

  test('Find in Plan navigates from Do view to Plan tree location', async ({
    plan,
  }) => {
    await test.step('Verify task visible in Do view', async () => {
      await plan.switchToDoView();
      await plan.verifyTaskVisible('Research Requirements');
    });

    await test.step('Execute Find in Plan', async () => {
      await plan.findInPlan('Research Requirements');
    });

    await test.step('Verify Plan view shows expanded hierarchy', async () => {
      await plan.verifyTaskVisible('Project Alpha');
      await plan.verifyTaskVisible('Research Requirements');
      await plan.verifyTaskVisible('Design UI Mocks');
    });
  });

  test('should edit task properties and persist changes', async ({
    plan,
    page,
  }) => {
    const taskTitle = 'Task to Edit';
    const newTitle = 'Edited Task Title';

    await test.step('Create and edit task', async () => {
      await plan.createTask(taskTitle);
      await plan.editTaskTitle(taskTitle, newTitle);
      await plan.verifyTaskVisible(newTitle);
      await plan.verifyTaskHidden(taskTitle);
    });

    await test.step('Verify persistence after reload', async () => {
      await page.reload();
      await plan.switchToPlanView();
      await plan.verifyTaskVisible(newTitle);
    });
  });

  test('should delete task with cascade', async ({ plan }) => {
    const parentTitle = 'Parent Task';
    const childTitle = 'Child Task';

    await test.step('Setup hierarchy', async () => {
      // Given a hierarchy: Parent -> Child
      await plan.createTask(parentTitle);
      await plan.openTaskEditor(parentTitle);
      await plan.addChild(childTitle);
    });

    await test.step('Delete parent with cascade', async () => {
      // When the user deletes the parent
      await plan.deleteTask(parentTitle);

      // Then both parent and child are removed
      await plan.verifyTaskHidden(parentTitle);
      await plan.verifyTaskHidden(childTitle);
    });
  });

  test('should persist data across page reloads', async ({ plan, page }) => {
    const persistTask = 'Persistent Task';

    await test.step('Create task and reload', async () => {
      // Given a task is created
      await plan.createTask(persistTask);

      // When the page is reloaded
      await page.reload();
      await plan.switchToPlanView();

      // Then the task is still visible
      await plan.verifyTaskVisible(persistTask);
    });
  });
});
