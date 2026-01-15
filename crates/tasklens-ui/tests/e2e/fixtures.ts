import { expect, type Page } from "@playwright/test";
import { test as bddTest } from "playwright-bdd";
import { type PlanFixture, PlanPage } from "./pages/plan-page";
import { dumpFailureContext } from "./utils/debug-utils";

export { expect };

type DocumentContextFixture = {
  documentContext: {
    documents: Map<string, string>;
  };
};

// Combine all fixtures
type MyFixtures = {
  plan: PlanFixture;
  debugFailure: null;
} & DocumentContextFixture;

export const test = bddTest.extend<MyFixtures>({
  plan: async (
    { page }: { page: Page },
    use: (r: PlanPage) => Promise<void>,
  ) => {
    const planPage = new PlanPage(page);
    await use(planPage);
  },
  documentContext: async (
    // biome-ignore lint/correctness/noEmptyPattern: playwright-bdd requires destructuring pattern
    {},
    use: (r: { documents: Map<string, string> }) => Promise<void>,
  ) => {
    await use({ documents: new Map() });
  },
  debugFailure: [
    async (
      { page }: { page: Page },
      use: (r: null) => Promise<void>,
      testInfo: import("@playwright/test").TestInfo,
    ) => {
      await use(null);
      if (testInfo.status !== "passed" && testInfo.status !== "skipped") {
        await dumpFailureContext(page, testInfo);
      }
    },
    { auto: true },
  ],
});
