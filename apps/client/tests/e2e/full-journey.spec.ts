import {expect, test} from '@playwright/test';

/**
 * Full User Journey: Desktop
 *
 * This test suite validates the core "happy path" for desktop users, starting from an empty state
 * and building out a complex task hierarchy. It verifies:
 * - Application initialization and navigation to the Plan view.
 * - Initial task creation from the empty state.
 * - Task editing and rename persistence.
 * - Hierarchical structure management (Add Sibling, Add Child).
 * - UI feedback and focus management during structurally transformative actions.
 */
test.describe('Full User Journey: Desktop', () => {
  test.use({viewport: {width: 1280, height: 720}});

  test('Start Empty -> Add -> Edit -> Sibling -> Child', async ({
    page,
  }, testInfo) => {
    // Debug: Listen to logs
    page.on('console', msg => console.log(`BROWSER LOG: ${msg.text()}`));

    // 1. Start Empty
    await page.goto('/');

    try {
      // Wait for app skeleton
      await expect(page.getByRole('heading', {name: 'Mydoo'})).toBeVisible({
        timeout: 15000,
      });

      // Explicitly wait for loader to vanish
      await expect(page.locator('.mantine-LoadingOverlay-root')).toBeHidden({
        timeout: 15000,
      });

      // Take screenshot of initial state
      await page.screenshot({path: `test-results/initial-state.png`});

      // Navigate to Plan view (default is Do)
      // Note: 'isMobile' is not defined in this scope. Assuming desktop for this test.
      await page.getByRole('button', {name: 'Plan'}).click();

      const addFirstBtn = page.getByRole('button', {name: 'Add First Task'});
      const taskItem = page.getByTestId('task-item').first();

      // Wait for either the button OR a task to be visible, establishing loaded state
      try {
        await expect(addFirstBtn.or(taskItem)).toBeVisible({timeout: 15000});
      } catch (e) {
        console.log('TIMEOUT: Dumping page content');
        const content = await page.content();
        console.log(content);
        throw e;
      }

      if (await addFirstBtn.isVisible()) {
        await addFirstBtn.click();
        await expect(
          page.getByRole('heading', {name: 'Create Task'}),
        ).toBeVisible();
        // Verify Title is focused (Desktop)
        await expect(page.getByRole('textbox', {name: 'Title'})).toBeFocused();
        await page.getByRole('textbox', {name: 'Title'}).fill('My First Task');
        await page.getByRole('button', {name: 'Create Task'}).click();
      }

      // 2. Rename Root Task
      const firstTask = page.getByTestId('task-item').first();
      await expect(firstTask).toBeVisible();
      await firstTask.getByText(/./).first().click(); // Click text to edit
      await expect(page.getByRole('dialog')).toBeVisible();
      await page.getByRole('textbox', {name: 'Title'}).fill('Desktop Root');
      await page.getByRole('button', {name: 'Save Changes'}).click();

      await expect(firstTask).toContainText('Desktop Root');

      // 3. Add Sibling (via Hover Menu)
      // Note: Hover is tricky, but clicking the trigger works if visible.
      // The menu trigger is always rendered but might depend on CSS for visibility?
      // The CSS has `.task-menu-trigger { opacity: 0; }` and `:hover { opacity: 1; }`.
      // Playwright can click invisible elements if forced, or we hover first.
      await firstTask.hover();
      await firstTask.getByTestId('task-menu-trigger').click();
      await page.getByRole('menuitem', {name: 'Add Sibling'}).click();

      await expect(page.getByRole('dialog')).toBeVisible();
      await page.getByRole('textbox', {name: 'Title'}).fill('Desktop Sibling');
      await page.getByRole('button', {name: 'Create Task'}).click();

      // Verify presence and order
      await expect(page.getByText('Desktop Sibling')).toBeVisible();
      const itemsAfterSibling = page.getByTestId('task-item');
      await expect(itemsAfterSibling.nth(0)).toContainText('Desktop Root');
      await expect(itemsAfterSibling.nth(1)).toContainText('Desktop Sibling');

      // 4. Add Child (via Hover Menu on Root)
      await itemsAfterSibling.nth(0).hover();
      await itemsAfterSibling.nth(0).getByTestId('task-menu-trigger').click();
      await page.getByRole('menuitem', {name: 'Add Child'}).click();

      await expect(page.getByRole('dialog')).toBeVisible();
      await page.getByRole('textbox', {name: 'Title'}).fill('Desktop Child');
      await page.getByRole('button', {name: 'Create Task'}).click();

      // Verify hierarchy
      // Auto-expand is now handled by Step 6E (implemented).

      // Wait for modal to close
      await expect(page.getByRole('dialog')).toBeHidden();

      // "Desktop Child" should be visible automatically due to auto-expand.
      await expect(page.getByText('Desktop Child')).toBeVisible();

      // Optional: Verify indentation or order (Child comes after Parent)
      const itemsAfterChild = page.getByTestId('task-item');
      await expect(itemsAfterChild.nth(1)).toContainText('Desktop Child'); // Child should be strictly after parent if parent has no other children
      await expect(itemsAfterChild.nth(2)).toContainText('Desktop Sibling'); // Sibling is pushed down

      // ... rest of test ...
    } catch (e) {
      // Capture screenshot on failure
      await page.screenshot({
        path: `test-results/failure-${testInfo.title}.png`,
      });
      throw e;
    }
  });
});
