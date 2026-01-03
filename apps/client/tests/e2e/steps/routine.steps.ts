import {createBdd} from 'playwright-bdd';
import {test} from '../fixtures';

const {Given, When, Then} = createBdd(test);

Given(
  'the user creates a routine task {string} repeating every {string} with lead time {string}',
  async ({plan}, title, repeatStr, leadTimeStr) => {
    await plan.setupClock();
    await plan.primeWithSampleData();
    // Parse duration strings (e.g., "2 days", "8 hours")
    // Simple parser for now
    const parseDuration = (str: string) => {
      const parts = str.split(' ');
      const val = parseInt(parts[0] || '0', 10);
      const unit = parts[1]?.toLowerCase() || '';
      // Unit mapping to UI expectations or logic
      // UI has "Minutes", "Hours", "Days"
      let uiUnit = 'Days';
      if (unit.startsWith('minute')) uiUnit = 'Minutes';
      if (unit.startsWith('hour')) uiUnit = 'Hours';
      if (unit.startsWith('day')) uiUnit = 'Days';

      return {val, uiUnit, rawUnit: unit};
    };

    const repeat = parseDuration(repeatStr);
    const lead = parseDuration(leadTimeStr);

    // Map repeat unit to frequency value expected by UI Select
    // UI data: 'minutes', 'hours', 'daily', 'weekly', 'monthly', 'yearly'
    // Map "Days" -> "daily" if interval is 1? Or does UI support "Every 2 Days"?
    // Looking at TaskEditorModal:
    // data={[ {value: 'minutes', label: 'Minutes'}, {value: 'hours', label: 'Hours'}, {value: 'daily', label: 'Daily'} ... ]}
    // and "Every X units" input appears if frequency is set.
    // If it's "2 days", we might need to select "Daily" and interval 2.
    // Wait, the UI has "Daily" which implies 1 day.
    // Does it support interval for Daily?
    // Code says: {frequency && ( <NumberInput label="Every X units" ... /> )}
    // So yes, if frequency is selected, interval input is shown.

    let freqValue = 'daily';
    if (repeat.uiUnit === 'Minutes') freqValue = 'minutes';
    if (repeat.uiUnit === 'Hours') freqValue = 'hours';
    if (repeat.uiUnit === 'Days') freqValue = 'daily';
    // TODO: Handle weeks/months if needed, but test only uses days/hours/minutes

    await plan.createRoutineTask(title, {
      frequency: freqValue,
      interval: repeat.val,
      leadTimeVal: lead.val,
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
