import { type Browser, expect, type Page } from "@playwright/test";
import { test as bddTest } from "playwright-bdd";
import { type PlanFixture, PlanPage } from "./pages/plan-page";
import { Steps } from "./steps/steps-library";
import { dumpFailureContext, formatConsoleMessage } from "./utils/debug-utils";
import { SyncServerHelper } from "./utils/sync-server";

export { expect };

type UserContext = {
  page: Page;
  plan: PlanPage;
};

type DocumentContextFixture = {
  documentContext: {
    documents: Map<string, string>;
  };
};

// Combine all fixtures
type MyFixtures = {
  plan: PlanFixture;
  debugFailure: null;
  I: Steps;
  alice: UserContext;
  bob: UserContext;
} & DocumentContextFixture;

type MyWorkerFixtures = {
  syncServer: SyncServerHelper;
};

const createUserFixture = async (
  browser: Browser,
  name: string,
): Promise<UserContext> => {
  const context = await browser.newContext({ serviceWorkers: "block" });
  const page = await context.newPage();
  const plan = new PlanPage(page);

  page.on("console", async (msg) => {
    const type = msg.type();
    const text = await formatConsoleMessage(msg);
    console.log(`[${name}] PAGE ${type}: ${text}`);
  });

  return { page, plan };
};

export const test = bddTest.extend<MyFixtures, MyWorkerFixtures>({
  plan: async (
    { page }: { page: Page },
    use: (r: PlanPage) => Promise<void>,
  ) => {
    const planPage = new PlanPage(page);
    page.on("console", async (msg) => {
      const type = msg.type();
      const cleanText = await formatConsoleMessage(msg);
      const text = `PAGE ${type}: ${cleanText}`;
      if (type === "error") {
        console.error(text);
      } else if (type === "warning") {
        console.warn(text);
      } else if (type === "debug") {
        console.debug(text);
      } else {
        console.log(text);
      }
    });
    await use(planPage);
  },
  alice: async ({ browser }, use) => {
    const user = await createUserFixture(browser, "Alice");
    await use(user);
    await user.page.context().close();
  },
  bob: async ({ browser }, use) => {
    const user = await createUserFixture(browser, "Bob");
    await use(user);
    await user.page.context().close();
  },
  I: async ({ plan, page }, use) => {
    // plan fixture is typed as interface but at runtime it's PlanPage instance
    // We cast to PlanPage because Steps expects the concrete class or compatible interface
    await use(new Steps(plan as PlanPage, page));
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
  syncServer: [
    async (
      // biome-ignore lint/correctness/noEmptyPattern: playwright fixture requirement
      {},
      use,
      workerInfo,
    ) => {
      const port = 3010 + workerInfo.workerIndex;
      const server = new SyncServerHelper(port);
      await server.start();
      await use(server);
      await server.stop();
    },
    { scope: "worker" },
  ],
});
