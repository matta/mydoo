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
  fullyParallel: false,
  forbidOnly: isCI,
  retries: isCI ? 2 : 0,
  workers: 1,
  reporter: [["html", { open: "never" }]],
  use: {
    baseURL: "http://localhost:5179",
    trace: "on-first-retry",
    timezoneId: "Asia/Tokyo",
    locale: "en-US",
  },
  webServer: {
    command: "pnpm run dev --port 5179 --strictPort",
    url: "http://localhost:5179",
    reuseExistingServer: !isCI,
  },
  projects: [
    {
      name: "bdd-desktop",
      testDir: testDirBdd,
      use: { ...devices["Desktop Chrome"] },
      grepInvert: /@mobile/,
    },
    {
      name: "bdd-mobile",
      testDir: testDirBdd,
      use: { ...devices["Pixel 7"] },
      grep: /@mobile/,
    },
    {
      name: "e2e",
      testDir: "tests/e2e",
      testIgnore: ["features/**", "steps/**", ".features-gen/**"],
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
