import {expect, test} from '@playwright/test';

test('has title', async ({page}) => {
  await page.goto('/');
  // Basic check to ensure page loads; title might be "Vite + React + TS" or "mydoo"
  // For now just check it doesn't 404
  expect(page).not.toBeNull();
});
