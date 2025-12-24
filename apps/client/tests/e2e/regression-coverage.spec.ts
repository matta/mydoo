import {expect, test} from '@playwright/test';

/**
 * Regression Coverage: Full Journey
 *
 * This suite verifies the complete end-to-end user workflows for both Desktop and Mobile viewports.
 * It serves as a quality gate for critical creation and navigation flows, ensuring that:
 * - New tasks are created in the correct position (sibling/child).
 * - Auto-focus stability is maintained in the Task Editor Modal for both Create and Edit modes.
 * - Hierarchy interactions keep tasks visible via Auto-Expand (Desktop) and Auto-Drill (Mobile).
 * - Focus management is robust across multiple modal open/close cycles and menu interactions.
 */
test.describe('Regression Coverage: Full Journey', () => {
  test.describe('Desktop View', () => {
    test.use({viewport: {width: 1280, height: 720}});

    test('Complete Desktop Journey', async ({page}) => {
      await page.goto('/');
      await expect(page.getByRole('heading', {name: 'Mydoo'})).toBeVisible({
        timeout: 10000,
      });
      await page.getByRole('button', {name: 'Plan'}).click();

      // 1. Start Empty
      const addFirstBtn = page.getByRole('button', {name: 'Add First Task'});
      await expect(addFirstBtn).toBeVisible();
      await addFirstBtn.click();

      // 2. Verify Auto-focus on Create Task
      await expect(
        page.getByRole('heading', {name: 'Create Task'}),
      ).toBeVisible();
      const titleInput = page.getByRole('textbox', {name: 'Title'});
      await expect(titleInput).toBeFocused({timeout: 10000});

      // 3. Create Root Task
      const rootTitle = 'Desktop Root';
      await titleInput.fill(rootTitle);
      await page.getByRole('button', {name: 'Create Task'}).click();
      await expect(page.getByTestId('task-item')).toContainText(rootTitle);

      // 4. Rename Root Task (Verify Auto-focus on Edit)
      const firstTask = page.getByTestId('task-item').first();
      await firstTask.getByText(rootTitle).click();
      await expect(
        page.getByRole('heading', {name: 'Edit Task'}),
      ).toBeVisible();
      await expect(titleInput).toBeFocused({timeout: 10000});
      const newRootTitle = 'Desktop Root Renamed';
      await titleInput.fill(newRootTitle);
      await page.getByRole('button', {name: 'Save Changes'}).click();
      await expect(page.getByRole('dialog')).toBeHidden(); // Wait for modal to close
      await expect(page.getByTestId('task-item')).toContainText(newRootTitle);

      // 5. Add Sibling (Desktop Menu)
      await firstTask.hover();
      await firstTask.getByTestId('task-menu-trigger').click();
      await page.getByRole('menuitem', {name: 'Add Sibling'}).click();
      const siblingTitle = 'Desktop Sibling';
      await expect(titleInput).toBeFocused({timeout: 10000});
      await titleInput.fill(siblingTitle);
      await page.getByRole('button', {name: 'Create Task'}).click();
      await expect(page.getByRole('dialog')).toBeHidden(); // Wait for modal to close
      await expect(page.getByText(siblingTitle)).toBeVisible();

      // 6. Add Child (Desktop Menu)
      await page
        .getByTestId('task-item')
        .filter({hasText: newRootTitle})
        .hover();
      await page
        .getByTestId('task-item')
        .filter({hasText: newRootTitle})
        .getByTestId('task-menu-trigger')
        .click();
      await page.getByRole('menuitem', {name: 'Add Child'}).click();
      const childTitle = 'Desktop Child';
      await titleInput.fill(childTitle);
      await page.getByRole('button', {name: 'Create Task'}).click();

      // 7. Verify Child is visible (Auto-expand)
      await expect(page.getByText(childTitle)).toBeVisible();
    });
  });

  test.describe('Mobile View', () => {
    test.use({viewport: {width: 375, height: 667}});

    test('Complete Mobile Journey', async ({page}) => {
      await page.goto('/');
      await expect(page.getByRole('heading', {name: 'Mydoo'})).toBeVisible({
        timeout: 10000,
      });
      await page.getByRole('button', {name: 'Plan'}).last().click();

      // 1. Start Empty
      const addFirstBtn = page.getByRole('button', {name: 'Add First Task'});
      await expect(addFirstBtn).toBeVisible();
      await addFirstBtn.click();

      // 2. Verify Auto-focus
      await expect(
        page.getByRole('heading', {name: 'Create Task'}),
      ).toBeVisible();
      const titleInput = page.getByRole('textbox', {name: 'Title'});
      await expect(titleInput).toBeFocused({timeout: 10000});

      // 3. Create Root Task
      const rootTitle = 'Mobile Root';
      await titleInput.fill(rootTitle);
      await page.getByRole('button', {name: 'Create Task'}).click();
      await expect(page.getByTestId('task-item')).toContainText(rootTitle);

      // 4. Add Sibling (Footer Action)
      await page.getByLabel('Add Task at Top').click(); // Using Add Task at Top for mobile footer
      const siblingTitle = 'Mobile Sibling';
      await titleInput.fill(siblingTitle);
      await page.getByRole('button', {name: 'Create Task'}).click();
      await expect(page.getByRole('dialog')).toBeHidden(); // Wait for modal to close
      await expect(page.getByText(siblingTitle)).toBeVisible();

      // 5. Add Child (Context Menu -> Auto-drill)
      const rootRow = page
        .getByTestId('task-item')
        .filter({hasText: rootTitle})
        .first();
      await rootRow.getByLabel('Task actions').click();
      await page.getByRole('menuitem', {name: 'Add Child'}).click();
      const childTitle = 'Mobile Child';
      await titleInput.fill(childTitle);
      await page.getByRole('button', {name: 'Create Task'}).click();

      // 6. Verify Auto-drill
      await expect(page.getByRole('button', {name: rootTitle})).toBeVisible(); // Breadcrumb
      await expect(page.getByText(childTitle)).toBeVisible();
    });
  });
});
