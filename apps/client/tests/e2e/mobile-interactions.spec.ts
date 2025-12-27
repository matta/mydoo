import {test} from './fixtures';

test.describe('Mobile Interactions', () => {
  // iPhone SE viewport
  test.use({viewport: {width: 375, height: 667}});

  test.beforeEach(async ({page, plan}) => {
    await page.goto('/?seed=true');
    await plan.switchToPlanView();
  });

  test('Mobile Smoke Test', async ({plan}) => {
    await test.step('Verify Bottom Bar', async () => {
      await plan.mobileVerifyMobileBottomBar();
    });
  });

  test('Add Child via Drill Down', async ({plan}) => {
    const parentTitle = 'Deep Work Project'; // From Seed Data
    const childTitle = `Drill Child ${Date.now()}`;

    await test.step('Drill Down', async () => {
      await plan.mobileDrillDown(parentTitle);
      await plan.mobileVerifyViewTitle(parentTitle);
    });

    await test.step('Create Child', async () => {
      await plan.createTask(childTitle);
      await plan.verifyTaskVisible(childTitle);
    });

    await test.step('Go Back and Verify Root View', async () => {
      await plan.mobileNavigateUpLevel();
      // After going back from parent, we should be at the root Plan view.
      // Verify the bottom bar is visible (root indicator for mobile).
      await plan.mobileVerifyMobileBottomBar();
    });
  });
});
