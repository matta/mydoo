import {expect, test} from '@playwright/test';

test.describe('Priority List', () => {
  test('seed data loads and renders tasks', async ({page}) => {
    // 1. Visit the app with ?seed=true to populate data
    await page.goto('/?seed=true');

    // 2. Wait for the list container to appear
    const listContainer = page.getByRole('heading', {name: 'Priorities'});
    await expect(listContainer).toBeVisible();

    // 3. Verify the tasks are rendered in the correct order (Priority/Importance ID sort)
    // Seeding: Buy Milk (1), Walk Dog (0.5), Read Book (0.1)
    const taskRows = page.getByRole('checkbox');
    await expect(taskRows).toHaveCount(3);

    // Verify content text
    // Note: We rely on unit tests for precise sorting verification.
    // Here we primarily verify that seeded tasks are successfully rendered and visible.
    await expect(page.getByText('Buy Milk')).toBeVisible();
    await expect(page.getByText('Walk Dog')).toBeVisible();
    await expect(page.getByText('Read Book')).toBeVisible();
  });
});
