import {expect, test} from '@playwright/test';

test.describe('Task Interactions', () => {
  test.beforeEach(async ({page}) => {
    // Start with a clean slate (no seed) or use a specific seed for predictable ID generation if needed
    // For interactions, starting fresh is usually fine
    await page.goto('/?seed=true');
  });

  test('create and complete a task', async ({page}) => {
    const taskName = 'New E2E Task';

    // 1. Create a task
    const input = page.getByPlaceholder('Add a new task...');
    await expect(input).toBeVisible();
    await input.fill(taskName);
    await input.press('Enter');

    // 2. Verify it appears
    const taskRow = page.getByText(taskName);
    await expect(taskRow).toBeVisible();

    // 3. Complete the task
    // Use .click() instead of .check() because the element is removed from the DOM
    // immediately upon completion (filtered out), preventing .check() from verifying the "checked" state.
    const checkbox = page.getByRole('checkbox', {
      name: `Complete ${taskName}`,
    });
    await checkbox.click();

    // 4. Verify it remains visible (strikethrough state) until acknowledged
    await expect(taskRow).toBeVisible();
    await expect(taskRow).toHaveCSS('text-decoration-line', 'line-through');

    // 5. Click Refresh to acknowledge and clear
    const refreshButton = page.getByRole('button', {name: 'Refresh'});
    await refreshButton.click();

    // 6. Verify it disappears
    await expect(taskRow).toBeHidden();
  });
});
