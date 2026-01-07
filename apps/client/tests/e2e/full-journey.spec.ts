import { test } from './fixtures';

test.describe('Full User Journey: Desktop', () => {
  test.use({ viewport: { width: 1280, height: 720 } });

  test('Start Empty -> Add -> Edit -> Sibling -> Child', async ({
    plan,
    page,
  }) => {
    // 1. Start Empty
    await test.step('Initial state and navigation', async () => {
      await page.goto('/');
      await plan.switchToPlanView();
    });

    await test.step('Create first task', async () => {
      const rootTitle = 'Desktop Root';
      await plan.addFirstTask(rootTitle);
      await plan.verifyTaskVisible(rootTitle);
    });

    await test.step('Rename task', async () => {
      const rootTitle = 'Desktop Root';
      const renamedTitle = 'Desktop Root Renamed';
      await plan.editTaskTitle(rootTitle, renamedTitle);
      await plan.verifyTaskVisible(renamedTitle);
      await plan.verifyTaskHidden(rootTitle);
    });

    await test.step('Add sibling', async () => {
      const rootTitle = 'Desktop Root Renamed';
      const siblingTitle = 'Desktop Sibling';
      await plan.addSibling(rootTitle, siblingTitle);
      await plan.verifyTaskVisible(rootTitle);
      await plan.verifyTaskVisible(siblingTitle);
    });

    await test.step('Add child and verify hierarchy', async () => {
      const rootTitle = 'Desktop Root Renamed';
      const childTitle = 'Desktop Child';

      await plan.openTaskEditor(rootTitle);
      await plan.addChild(childTitle);

      await plan.verifyTaskVisible(childTitle);
    });
  });
});
