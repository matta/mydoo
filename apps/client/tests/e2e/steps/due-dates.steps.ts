import {createBdd} from 'playwright-bdd';
import {formatDateAsISO} from '../../../src/test/utils/date-formatter';
import {
  durationToMs,
  parseDuration,
} from '../../../src/test/utils/duration-parser';
import {expect, test} from '../fixtures';

const {Given} = createBdd(test);

Given(
  'the user creates a task {string} with due date {string} from now and lead time {string}',
  async ({plan, page}, title, dueStr, leadTimeStr) => {
    // Setup: Install clock and start with clean state
    await plan.setupClock();
    await page.goto('/');
    await page.evaluate(() => localStorage.clear());
    await page.reload();
    // setupClock() must be called again after reload because page reload resets the clock
    await plan.setupClock();

    // Wait for app to be ready
    await expect(
      page.locator('nav, footer').getByRole('button', {name: 'Plan'}).last(),
    ).toBeVisible();

    // Parse lead time duration
    const leadDuration = parseDuration(leadTimeStr);

    // Calculate due date as YYYY-MM-DD string
    const now = new Date(await page.evaluate(() => Date.now()));
    const dueMs = durationToMs(dueStr);
    now.setTime(now.getTime() + dueMs);
    const dateString = formatDateAsISO(now);

    // Create task with due date via Page Object
    await plan.createTaskWithDueDate(title, {
      dueDate: dateString,
      leadTimeVal: leadDuration.value,
      leadTimeUnit: leadDuration.uiUnit,
    });
  },
);

// Note: 'Then the task {string} should be visible/hidden in the Do list' steps
// are defined in routine.steps.ts and shared across features.
