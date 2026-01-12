import { defineConfig, devices } from "@playwright/test";
import { defineBddConfig } from "playwright-bdd";

const bddCommon = {
  features: "tests/e2e/features/*.feature",
  steps: "tests/e2e/{steps/*.steps.ts,fixtures.ts}",
};

const testDirDesktop = defineBddConfig({
  ...bddCommon,
  outputDir: "tests/e2e/.features-gen/desktop",
});

const testDirMobile = defineBddConfig({
  ...bddCommon,
  outputDir: "tests/e2e/.features-gen/mobile",
});

const isCI = !!process.env.CI;

export default defineConfig({
  fullyParallel: true,
  forbidOnly: isCI,
  retries: isCI ? 2 : 0,
  ...(isCI && { workers: 4 }),
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
      testDir: testDirDesktop,
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "bdd-mobile",
      testDir: testDirMobile,
      use: { ...devices["Pixel 7"] },
      grepInvert: /@skip-mobile/,
    },
    {
      name: "e2e",
      testDir: "tests/e2e",
      testIgnore: ["features/**", "steps/**", ".features-gen/**"],
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
