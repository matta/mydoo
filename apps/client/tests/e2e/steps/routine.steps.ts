import {createBdd} from 'playwright-bdd';
import {parseDuration} from '../../../src/test/utils/duration-parser';
import {test} from '../fixtures';

const {Given, When, Then} = createBdd(test);

Given(
  'the user creates a routine task {string} repeating every {string} with lead time {string}',
  async ({plan}, title, repeatStr, leadTimeStr) => {
    await plan.setupClock();
    await plan.primeWithSampleData();

    const repeat = parseDuration(repeatStr);
    const lead = parseDuration(leadTimeStr);

    // Map UI unit to frequency value expected by the Select component
    const freqMap: Record<string, string> = {
      Minutes: 'minutes',
      Hours: 'hours',
      Days: 'daily',
    };
    const freqValue = freqMap[repeat.uiUnit];
    if (!freqValue) {
      throw new Error(
        `Unsupported repeat unit: "${repeat.uiUnit}". Expected: Minutes, Hours, or Days.`,
      );
    }

    await plan.createRoutineTask(title, {
      frequency: freqValue,
      interval: repeat.value,
      leadTimeVal: lead.value,
      leadTimeUnit: lead.uiUnit,
    });
  },
);

Then(
  'the task {string} should be visible in the Do list',
  async ({plan}, title) => {
    await plan.switchToDoView();
    await plan.verifyTaskVisible(title);
  },
);

When(
  'the user completes the task {string} from the Do list',
  async ({plan}, title) => {
    await plan.switchToDoView();
    await plan.completeTask(title);
  },
);

Then(
  'the task {string} should be marked as completed in the Do list',
  async ({plan}, title) => {
    await plan.switchToDoView();
    await plan.verifyTaskCompleted(title);
  },
);

When('the user refreshes the Do list', async ({plan}) => {
  await plan.switchToDoView();
  await plan.refreshDoList();
});

Then(
  'the task {string} should be hidden in the Do list',
  async ({plan}, title) => {
    await plan.switchToDoView();
    await plan.verifyTaskHidden(title);
  },
);

When('the user waits {string}', async ({plan}, durationStr) => {
  const parts = durationStr.split(' ');
  const val = parseInt(parts[0] || '0', 10);
  const unit = parts[1]?.toLowerCase() || '';

  let minutes = 0;
  if (unit.startsWith('minute')) minutes = val;
  else if (unit.startsWith('hour')) minutes = val * 60;
  else if (unit.startsWith('day')) minutes = val * 60 * 24;

  await plan.advanceTime(minutes);
});
