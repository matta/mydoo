import { defineConfig, devices } from "@playwright/test";
import { defineBddConfig } from "playwright-bdd";

const bddCommon = {
  features: "tests/e2e/features/*.feature",
  steps: "tests/e2e/{steps/*.steps.ts,fixtures.ts}",
};

const testDirBdd = defineBddConfig({
  ...bddCommon,
  outputDir: "tests/e2e/.features-gen",
});

const isCI = !!process.env.CI;

export default defineConfig({
  workers: 1,
  reporter: [["html", { open: "never" }]],
  use: {
    baseURL: "http://localhost:5180",
    trace: "on-first-retry",
    timezoneId: "Asia/Tokyo",
    locale: "en-US",
    screenshot:
      (process.env.SCREENSHOT as "off" | "on" | "only-on-failure") || "off",
  },
  webServer: {
    command:
      "pnpm dlx serve ../../target/dx/tasklens-ui/debug/web/public -p 5180 -s",
    url: "http://localhost:5180",
    reuseExistingServer: !isCI,
  },
  projects: [
    {
      name: "bdd-desktop",
      testDir: testDirBdd,
      use: { ...devices["Desktop Chrome"] },
      grepInvert: /@migration-pending/,
    },
    {
      name: "bdd-mobile",
      testDir: testDirBdd,
      use: { ...devices["Pixel 7"] },
      grepInvert: /@migration-pending/,
    },
    {
      name: "e2e-desktop",
      testDir: "tests/e2e",
      testIgnore: ["features/**", "steps/**", ".features-gen/**"],
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "e2e-mobile",
      testDir: "tests/e2e",
      testIgnore: ["features/**", "steps/**", ".features-gen/**"],
      use: { ...devices["Pixel 7"] },
    },
  ],
});
