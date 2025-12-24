import {expect, test} from '@playwright/test';

test.describe('Mobile Bottom Bar Interactions', () => {
  // iPhone SE viewport
  test.use({viewport: {width: 375, height: 667}});

  test.describe('Empty State', () => {
    test.beforeEach(async ({page}) => {
      await page.goto('/');
      await expect(page.getByRole('heading', {name: 'Mydoo'})).toBeVisible({
        timeout: 10000,
      });
      await page.getByRole('button', {name: 'Plan'}).last().click();
    });

    test('Shows "Add First Task" and allows creation', async ({page}) => {
      // Check for empty state message
      await expect(page.getByText('No tasks found.')).toBeVisible();

      // Click "Add First Task" (center button)
      await page.getByRole('button', {name: 'Add First Task'}).click();

      // Create
      await expect(
        page.getByRole('heading', {name: 'Create Task'}),
      ).toBeVisible();
      const title = `First Task ${Date.now()}`;
      await page.getByRole('textbox', {name: 'Title'}).fill(title);
      await page.getByRole('button', {name: 'Create Task'}).click();

      // Verify
      await expect(page.getByTestId('task-item').first()).toContainText(title);
    });
  });

  test.describe('Populated List Interactions', () => {
    test.beforeEach(async ({page}) => {
      await page.goto('/');
      await expect(page.getByRole('heading', {name: 'Mydoo'})).toBeVisible({
        timeout: 10000,
      });
      await page.getByRole('button', {name: 'Plan'}).last().click();

      // Ensure data exists via Seed if empty
      const noTasks = page.getByText('No tasks found.');
      if (await noTasks.isVisible()) {
        const devButton = page.getByRole('button', {name: 'Dev'});
        if (await devButton.isVisible()) {
          await devButton.click();
          await page.getByText('Seed Data').click();
          await expect(
            page.getByText('Deep Work Project', {exact: false}),
          ).toBeVisible();
        } else {
          // Fallback
          await page.getByRole('button', {name: 'Add First Task'}).click();
          await page.getByRole('textbox', {name: 'Title'}).fill('Seed Task');
          await page.getByRole('button', {name: 'Create Task'}).click();
        }
      }
      // Ensure list is visible
      await expect(page.getByTestId('task-item').first()).toBeVisible();
    });

    test('Bottom Bar UI Elements are present', async ({page}) => {
      await expect(page.getByLabel('Add Task at Top')).toBeVisible();
      await expect(page.getByLabel('Up Level')).toBeVisible();
      await expect(page.getByTestId('append-row-button')).toBeVisible();
    });

    test('Add to Top via Bottom Bar', async ({page}) => {
      await page.getByLabel('Add Task at Top').click();
      await expect(
        page.getByRole('heading', {name: 'Create Task'}),
      ).toBeVisible();
      // Verify Title is focused (Bug Fix)
      await expect(page.getByRole('textbox', {name: 'Title'})).toBeFocused();
      const title = `Top Task ${Date.now()}`;
      await page.getByRole('textbox', {name: 'Title'}).fill(title);
      await page.getByRole('button', {name: 'Create Task'}).click();
      await expect(page.getByTestId('task-item').first()).toContainText(title);
    });

    test('Add to Bottom via Append Row', async ({page}) => {
      await page.getByTestId('append-row-button').click();
      await expect(
        page.getByRole('heading', {name: 'Create Task'}),
      ).toBeVisible();
      const title = `Bottom Task ${Date.now()}`;
      await page.getByRole('textbox', {name: 'Title'}).fill(title);
      await page.getByRole('button', {name: 'Create Task'}).click();
      await expect(page.getByTestId('task-item').last()).toContainText(title);
    });

    test('Add Child via Drill Down', async ({page}) => {
      // 1. Create Parent
      await page.getByLabel('Add Task at Top').click();
      await page.getByRole('heading', {name: 'Create Task'}).waitFor();
      const parentTitle = `Parent ${Date.now()}`;
      await page.getByRole('textbox', {name: 'Title'}).fill(parentTitle);
      await page.getByRole('button', {name: 'Create Task'}).click();

      // 2. Drill down
      const parentRow = page
        .getByTestId('task-item')
        .filter({hasText: parentTitle})
        .first();
      await parentRow.getByLabel('Drill down').click();

      // 3. Verify Breadcrumb
      await expect(page.getByRole('button', {name: parentTitle})).toBeVisible();

      // 4. Add Task (should be child)
      await page.getByLabel('Add Task at Top').click();
      await page.getByRole('heading', {name: 'Create Task'}).waitFor();
      const childTitle = `Child ${Date.now()}`;
      await page.getByRole('textbox', {name: 'Title'}).fill(childTitle);
      await page.getByRole('button', {name: 'Create Task'}).click();

      // 5. Verify Child is visible in this view
      await expect(
        page.getByTestId('task-item').filter({hasText: childTitle}),
      ).toBeVisible();

      // 6. Go Up and verify Child is NOT visible in root
      await page.getByLabel('Up Level').click();
      await expect(
        page.getByTestId('task-item').filter({hasText: childTitle}),
      ).not.toBeVisible();
    });

    test('Add Child via Context Menu', async ({page}) => {
      // 1. Create Parent
      await page.getByLabel('Add Task at Top').click();
      await page.getByRole('heading', {name: 'Create Task'}).waitFor();
      const parentTitle = `Menu Parent ${Date.now()}`;
      await page.getByRole('textbox', {name: 'Title'}).fill(parentTitle);
      await page.getByRole('button', {name: 'Create Task'}).click();

      // 2. Open Context Menu
      const parentRow = page
        .getByTestId('task-item')
        .filter({hasText: parentTitle})
        .first();
      // On mobile, the menu trigger should now be visible or interactable
      // Assuming 'Task actions' label from TaskOutlineItem.tsx
      await parentRow.getByLabel('Task actions').click();

      // 3. Click "Add Child"
      await page.getByRole('menuitem', {name: 'Add Child'}).click();

      // 4. Create Child
      await expect(
        page.getByRole('heading', {name: 'Create Task'}),
      ).toBeVisible();
      const childTitle = `Menu Child ${Date.now()}`;
      await page.getByRole('textbox', {name: 'Title'}).fill(childTitle);
      await page.getByRole('button', {name: 'Create Task'}).click();

      // 5. Verify Child is added (might need to expand or drill down to see it?)
      // In 'drill' mode, adding a child to a node usually doesn't show it unless we are IN the parent.
      // BUT if we are at Root, and we added a child to Root Node, it becomes hidden under the parent.
      // We should see the parent has children indication (e.g. drill down arrow).
      // Or we can drill down to verify.
      await parentRow.getByLabel('Drill down').click();
      await expect(
        page.getByTestId('task-item').filter({hasText: childTitle}),
      ).toBeVisible();
    });
  });
});
