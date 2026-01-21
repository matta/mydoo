import { expect } from "@playwright/test";
import { test as bddTest } from "playwright-bdd";
import { type PlanFixture, PlanPage } from "./pages/plan-page";

export { expect };

type BinaryContext = {
  downloadedPath: string | null;
  originalDocId: string | null;
};

// Combine all fixtures
type MyFixtures = {
  plan: PlanFixture;
  documentContext: {
    documents: Map<string, string>;
  };
  binaryContext: BinaryContext;
};

export const test = bddTest.extend<MyFixtures>({
  plan: async ({ page }, use) => {
    const planPage = new PlanPage(page);
    await use(planPage);
  },
  documentContext: async (
    // biome-ignore lint/correctness/noEmptyPattern: playwright-bdd requires destructuring pattern
    {},
    use,
  ) => {
    await use({ documents: new Map() });
  },
  binaryContext: async (
    // biome-ignore lint/correctness/noEmptyPattern: playwright-bdd requires destructuring pattern
    {},
    use,
  ) => {
    await use({ downloadedPath: null, originalDocId: null });
  },
});
