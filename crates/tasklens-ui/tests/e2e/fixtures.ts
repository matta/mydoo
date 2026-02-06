import {
  type Browser,
  test as baseTest,
  expect,
  type Page,
  test as pwTest,
} from "@playwright/test";

import { type PlanFixture, PlanPage } from "./pages/plan-page";
import { Steps } from "./steps/steps-library";
import { dumpFailureContext, formatConsoleMessage } from "./utils/debug-utils";
import { SyncServerHelper } from "./utils/sync-server";

export { expect };

type UserContext = {
  page: Page;
  plan: PlanPage;
};

// Combine all fixtures
type MyFixtures = {
  plan: PlanFixture;
  debugFailure: null;
  I: Steps;
  alice: UserContext;
  bob: UserContext;
};

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

  if (process.env.SHOW_CONSOLE) {
    page.on("console", async (msg) => {
      const type = msg.type();
      const text = await formatConsoleMessage(msg);
      console.log(`[${name}] PAGE ${type}: ${text}`);
    });
  }

  return { page, plan };
};

export const test = baseTest.extend<MyFixtures, MyWorkerFixtures>({
  plan: async (
    { page }: { page: Page },
    use: (r: PlanPage) => Promise<void>,
  ) => {
    const planPage = new PlanPage(page);
    if (process.env.SHOW_CONSOLE) {
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
    }
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
  I: async ({ plan, page }, use, testInfo) => {
    // plan fixture is typed as interface but at runtime it's PlanPage instance
    // We cast to PlanPage because Steps expects the concrete class or compatible interface
    // Setup logic moved from onHomePage
    await plan.setupClock();
    await page.goto("/");
    await plan.waitForAppReady();

    const steps = new Steps(plan as PlanPage, page, testInfo);

    // Auto-wrap steps in test.step() for reporting
    const wrapStepGroup = (
      groupName: string,
      group: Record<string, unknown>,
    ) => {
      for (const key of Object.keys(group)) {
        const original = group[key];
        if (typeof original === "function") {
          group[key] = async (...args: unknown[]) => {
            // Convert camelCase to Title Case (e.g. "cleanWorkspace" -> "Clean Workspace")
            const title = key
              .replace(/([A-Z])/g, " $1")
              .replace(/^./, (str) => str.toUpperCase());

            // Format args for display if simple
            const argsStr = args.length
              ? ` ${args.map((a) => (typeof a === "string" ? `"${a}"` : JSON.stringify(a))).join(", ")}`
              : "";

            return await pwTest.step(
              `${groupName} ${title}${argsStr}`,
              async () => {
                // biome-ignore lint/suspicious/noExplicitAny: generic wrapper needs any
                return await original.apply(group, args as any[]);
              },
            );
          };
        }
      }
    };

    wrapStepGroup("Given", steps.Given);
    wrapStepGroup("When", steps.When);
    wrapStepGroup("Then", steps.Then);

    await use(steps);
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
