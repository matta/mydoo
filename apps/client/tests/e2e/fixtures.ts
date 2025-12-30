import {expect, type Page} from '@playwright/test';
import {test as bddTest} from 'playwright-bdd';
import {type PlanFixture, PlanPage} from './pages/plan-page';

export {expect};

type DocumentContextFixture = {
  documentContext: {
    previousDocId: string | undefined;
  };
};

// Combine all fixtures
type MyFixtures = {
  plan: PlanFixture;
} & DocumentContextFixture;

export const test = bddTest.extend<MyFixtures>({
  plan: async ({page}: {page: Page}, use: (r: PlanPage) => Promise<void>) => {
    const planPage = new PlanPage(page);
    await use(planPage);
  },
  documentContext: async (
    // biome-ignore lint/correctness/noEmptyPattern: playwright-bdd requires destructuring pattern
    {},
    use: (r: {previousDocId: string | undefined}) => Promise<void>,
  ) => {
    await use({previousDocId: undefined});
  },
});
