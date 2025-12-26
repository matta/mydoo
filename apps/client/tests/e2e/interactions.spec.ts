import {test} from './fixtures';

test.describe('Task Interactions', () => {
  test.beforeEach(async ({page}) => {
    await page.goto('/?seed=true');
  });

  test('create and complete a task', async ({plan}) => {
    const taskName = 'New E2E Task';

    await test.step('Create Task', async () => {
      await plan.createTask(taskName);
    });

    await test.step('Verify Created', async () => {
      await plan.verifyTaskVisible(taskName);
    });

    await test.step('Complete Task', async () => {
      await plan.completeTask(taskName);
      await plan.verifyTaskCompleted(taskName);
    });

    await test.step('Clear Completed Tasks', async () => {
      await plan.clearCompletedTasks();
    });

    await test.step('Verify Gone', async () => {
      await plan.verifyTaskHidden(taskName);
    });
  });
});
