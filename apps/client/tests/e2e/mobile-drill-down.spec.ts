import {expect, test} from '@playwright/test';

test.describe('Mobile Drill-Down Navigation', () => {
  // Use mobile viewport
  test.use({viewport: {width: 375, height: 667}});

  test.beforeEach(async ({page}) => {
    // Seed data (assuming seed utility is available or default state)
    // For now, we rely on default state or dev seeding if exposed.
    // Ideally we should use a clean slate or known state.
    // The app seems to start blank or with local storage.
    // We can use the creation UI to make structure if needed,
    // or assume we can inject data.
    // Let's assume we start fresh and create some tasks.
    await page.goto('/');

    // Switch to Plan View via mobile tab bar
    const planTab = page.getByRole('button', {name: 'Plan'}).last(); // Footer is last usually
    await expect(planTab).toBeVisible();
    await planTab.click();
  });

  test('Strict Viewport Mode: Mobile shows arrows, no chevrons', async ({
    page,
  }) => {
    // Seed Data
    const devButton = page.getByRole('button', {name: 'Dev'});
    if (await devButton.isVisible()) {
      await devButton.click();
      await page.getByText('Seed Data').click();
    } else {
      // Fallback: create manually if feasible, or skip if we can't seed
      console.log('Dev menu not found, skipping seed');
    }

    // Now we should have "Deep Work Project", etc.
    const rootTask = page
      .getByText('Deep Work Project', {exact: false})
      .first();
    await expect(rootTask).toBeVisible();

    // Check for icons
    // Drill Down arrow should be visible (IconArrowRight)
    const drillDownBtn = page.locator('[aria-label="Drill down"]').first();
    await expect(drillDownBtn).toBeVisible();

    // Chevron should NOT be visible
    const chevronBtn = page.locator('[aria-label="Toggle expansion"]').first();
    await expect(chevronBtn).not.toBeVisible();
  });

  test('Drill-down interaction', async ({page}) => {
    // Seed Data
    const devButton = page.getByRole('button', {name: 'Dev'});
    if (await devButton.isVisible()) {
      await devButton.click();
      await page.getByText('Seed Data').click();
    }

    // Find a parent task
    const parentRow = page
      .locator('[data-testid="task-item"]')
      .filter({hasText: 'Deep Work Project'})
      .first();

    // Click Drill Down
    await parentRow.locator('[aria-label="Drill down"]').click();

    // Verify view changed: Children should be visible (Module A)
    await expect(page.getByText('Module A')).toBeVisible();

    // Verify Parent is NOT visible in the list (it's in breadcrumbs)
    // The list shows children only.
    // Breadcrumb should show "Deep Work Project"
    await expect(
      page.getByRole('button', {name: 'Deep Work Project'}),
    ).toBeVisible();

    // Verify Back button (Up Level)
    await expect(page.getByLabel('Up Level')).toBeVisible();

    // Click Back
    await page.getByLabel('Up Level').click();

    // Verify back to root
    await expect(page.getByText('Deep Work Project')).toBeVisible();
    await expect(page.getByRole('button', {name: 'Back'})).not.toBeVisible();
  });

  test('4-level deep navigation', async ({page}) => {
    const devButton = page.getByRole('button', {name: 'Dev'});
    if (await devButton.isVisible()) {
      await devButton.click();
      await page.getByText('Seed Data').click();
    }

    // 1. Deep Work Project
    await page
      .locator('[data-testid="task-item"]')
      .filter({hasText: 'Deep Work Project'})
      .locator('[aria-label="Drill down"]')
      .click();

    // 2. Module A
    await page
      .locator('[data-testid="task-item"]')
      .filter({hasText: 'Module A'})
      .locator('[aria-label="Drill down"]')
      .click();

    // 3. Component X
    await page
      .locator('[data-testid="task-item"]')
      .filter({hasText: 'Component X'})
      .locator('[aria-label="Drill down"]')
      .click();

    // 4. (Leaf level or near leaf)
    // Should verify we are deep
    await expect(
      page.getByRole('button', {name: 'Deep Work Project'}),
    ).toBeVisible();
    await expect(page.getByRole('button', {name: 'Module A'})).toBeVisible();
    await expect(page.getByRole('button', {name: 'Component X'})).toBeVisible();
  });
});
