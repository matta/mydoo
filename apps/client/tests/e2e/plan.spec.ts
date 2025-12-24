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
});
