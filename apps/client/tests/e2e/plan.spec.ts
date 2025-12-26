import {expect, test} from '@playwright/test';

test.describe('Plan View', () => {
  test.beforeEach(async ({page}) => {
    await page.goto('/');
  });

  test('should seed data and render hierarchy', async ({page}) => {
    // 1. Trigger Seed Data via Dev Menu
    await page.getByRole('button', {name: 'Dev'}).click();
    await page.getByRole('menuitem', {name: 'Seed Data'}).click();

    // 2. Navigate to Plan tab
    await page.getByRole('button', {name: 'Plan'}).click();

    // 3. Verify top-level tasks are visible
    const alphaTask = page.getByText('Project Alpha');
    const groceryTask = page.getByText('Buy Groceries');

    await expect(alphaTask).toBeVisible();
    await expect(groceryTask).toBeVisible();

    // 4. Verify children are initially hidden (collapsed state)
    const researchTask = page.getByText('Research Requirements');
    await expect(researchTask).not.toBeVisible();

    // 5. Expand "Project Alpha"
    // Find the row containing "Project Alpha" and click the expand chevron
    const alphaRow = page.getByTestId('task-item').filter({has: alphaTask});
    await alphaRow.getByLabel('Toggle expansion').click();

    // 6. Verify children become visible
    await expect(researchTask).toBeVisible();
    await expect(page.getByText('Design UI Mocks')).toBeVisible();

    // Note: Breadcrumbs are hidden on desktop viewport per strict viewport modes.
    // Mobile breadcrumb behavior is tested in mobile-drill-down.spec.ts.
  });
  test('Find in Plan should navigate from Do view to Plan view tree location', async ({
    page,
  }) => {
    // 1. Seed Data
    await page.getByRole('button', {name: 'Dev'}).click();
    await page.getByRole('menuitem', {name: 'Seed Data'}).click();

    // 2. Go to Do Tab (ensure we are there)
    await page.getByRole('button', {name: 'Do'}).click();

    // 3. Open a child task ("Research Requirements")
    // Note: The Do view renders a flat list of tasks sorted by score.
    // "Research Requirements" is High Importance (1.0) so it should be visible.
    const task = page.getByText('Research Requirements');
    await expect(task).toBeVisible();
    await task.click();

    // 4. Click "Find in Plan" in the modal
    await page.getByRole('button', {name: 'Find in Plan'}).click();

    // 5. Verify Modal Closes
    await expect(page.getByRole('dialog')).not.toBeVisible();

    // 6. Verify we switched to Plan View
    // The Plan view renders the tree.
    // "Project Alpha" (parent) should be visible.
    const parent = page.getByText('Project Alpha');
    await expect(parent).toBeVisible();

    // 7. Verify Parent is Auto-Expanded
    // "Design UI Mocks" is a sibling of Research Requirements.
    // If the parent wasn't expanded, this sibling would be hidden.
    const sibling = page.getByText('Design UI Mocks');
    await expect(sibling).toBeVisible();

    // 8. Verify the Target Task is Visible
    await expect(page.getByText('Research Requirements')).toBeVisible();
  });
});
