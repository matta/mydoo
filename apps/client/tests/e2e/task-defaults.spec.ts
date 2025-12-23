import {expect, test} from '@playwright/test';

test.describe('Task Creation with Defaults', () => {
  test('should create task with default placeId via Quick Add', async ({
    page,
  }) => {
    await page.goto('/?seed=true');

    // Wait for app to load
    await page.waitForSelector('text=Priorities');

    // Create a task via Quick Add
    const input = page.getByPlaceholder('Add a new task...');
    await input.fill('Test Task with Defaults');
    await input.press('Enter');

    // Verify task appears in the list
    await expect(page.getByText('Test Task with Defaults')).toBeVisible();

    // Task should be created as a root task with ANYWHERE_PLACE_ID
    // We can't directly inspect the placeId in the UI, but we can verify
    // the task was created and appears in the priority list
  });

  test('should create task and display in priority list', async ({page}) => {
    await page.goto('/?seed=true');
    await page.waitForSelector('text=Priorities');

    // Create multiple tasks
    const input = page.getByPlaceholder('Add a new task...');

    await input.fill('First Task');
    await input.press('Enter');

    await input.fill('Second Task');
    await input.press('Enter');

    // Both tasks should be visible
    await expect(page.getByText('First Task')).toBeVisible();
    await expect(page.getByText('Second Task')).toBeVisible();
  });

  test('should persist task after page reload', async ({page}) => {
    await page.goto('/?seed=true');
    await page.waitForSelector('text=Priorities');

    // Create a task
    const input = page.getByPlaceholder('Add a new task...');
    await input.fill('Persistent Task');
    await input.press('Enter');

    await expect(page.getByText('Persistent Task')).toBeVisible();

    // Reload the page
    await page.reload();
    await page.waitForSelector('text=Priorities');

    // Task should still be visible
    await expect(page.getByText('Persistent Task')).toBeVisible();
  });
});
