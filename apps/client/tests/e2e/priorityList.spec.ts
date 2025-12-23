import {expect, test} from '@playwright/test';

test.describe('Priority List', () => {
  test('seed data loads and renders tasks', async ({page}) => {
    // 1. Visit the app with ?seed=true to populate data
    await page.goto('/?seed=true');

    // 2. Wait for the list container to appear
    const listContainer = page.getByRole('heading', {name: 'Priorities'});
    await expect(listContainer).toBeVisible();

    // 3. Verify leaf tasks are rendered
    // The Do View shows only leaf tasks (Pass 7 hides containers with visible children).
    // Seed data includes:
    // - Research Requirements, Design UI Mocks (children of Project Alpha)
    // - Milk, Eggs, Bread (children of Buy Groceries)
    // - Unit Test (deepest leaf)
    // - Quick Task (standalone)
    await expect(page.getByText('Research Requirements')).toBeVisible();
    await expect(page.getByText('Milk')).toBeVisible();
    await expect(page.getByText('Quick Task')).toBeVisible();
  });
});
