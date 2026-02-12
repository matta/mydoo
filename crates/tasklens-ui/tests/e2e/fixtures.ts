import fs from "node:fs";
import path from "node:path";
import {
  type Browser,
  type BrowserContext,
  test as baseTest,
  expect,
  type Page,
  test as pwTest,
} from "@playwright/test";

import { type PlanFixture, PlanPage } from "./pages/plan-page";
import { snapshotDir } from "./snapshot-paths";
import { Steps } from "./steps/steps-library";
import { dumpFailureContext, formatConsoleMessage } from "./utils/debug-utils";
import { getEphemeralPort, SyncServerHelper } from "./utils/sync-server";

export { expect };

type DbProfile = "empty" | "sample";

type UserContext = {
  page: Page;
  plan: PlanPage;
};

type MyFixtures = {
  db: DbProfile;
  plan: PlanFixture;
  debugFailure: null;
  I: Steps;
  alice: UserContext;
  bob: UserContext;
};

type MyWorkerFixtures = {
  syncServer: SyncServerHelper;
};

function snapshotPath(name: string): string {
  return path.join(snapshotDir, name);
}

function harPath(): string {
  return snapshotPath("app-assets.har");
}

function snapshotsExist(): boolean {
  return (
    fs.existsSync(snapshotPath("empty-db.json")) &&
    fs.existsSync(snapshotPath("sample-db.json")) &&
    fs.existsSync(harPath())
  );
}

async function applyHarReplay(context: BrowserContext): Promise<void> {
  const har = harPath();
  if (fs.existsSync(har)) {
    await context.routeFromHAR(har, { notFound: "fallback" });
  }
}

function attachConsoleLogger(page: Page, prefix?: string): void {
  if (!process.env.SHOW_CONSOLE) return;
  page.on("console", async (msg) => {
    const type = msg.type();
    const cleanText = await formatConsoleMessage(msg);
    const label = prefix ? `[${prefix}] ` : "";
    const text = `${label}PAGE ${type}: ${cleanText}`;
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

const createUserFixture = async (
  browser: Browser,
  name: string,
): Promise<UserContext> => {
  const context = await browser.newContext({ serviceWorkers: "block" });
  await applyHarReplay(context);
  const page = await context.newPage();
  const plan = new PlanPage(page);

  attachConsoleLogger(page, name);

  return { page, plan };
};

export const test = baseTest.extend<MyFixtures, MyWorkerFixtures>({
  db: ["empty", { option: true }],

  storageState: async ({ db }, use) => {
    if (snapshotsExist()) {
      await use(snapshotPath(`${db}-db.json`));
    } else {
      await use({ cookies: [], origins: [] });
    }
  },

  plan: async (
    { page, context }: { page: Page; context: BrowserContext },
    use: (r: PlanPage) => Promise<void>,
  ) => {
    await applyHarReplay(context);
    const planPage = new PlanPage(page);
    attachConsoleLogger(page);
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
  I: async ({ plan, page, syncServer, db }, use, testInfo) => {
    await plan.setupClock();
    await page.goto("/");
    await plan.waitForAppReady();

    const steps = new Steps(
      plan as PlanPage,
      page,
      testInfo,
      syncServer,
      db === "sample",
    );

    const wrapStepGroup = (
      groupName: string,
      group: Record<string, unknown>,
    ) => {
      for (const key of Object.keys(group)) {
        const original = group[key];
        if (typeof original === "function") {
          group[key] = async (...args: unknown[]) => {
            const title = key
              .replace(/([A-Z])/g, " $1")
              .replace(/^./, (str) => str.toUpperCase());

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
    ) => {
      const port = await getEphemeralPort();
      const server = new SyncServerHelper(port);
      await server.start();
      await use(server);
      await server.stop();
    },
    { scope: "worker" },
  ],
});
