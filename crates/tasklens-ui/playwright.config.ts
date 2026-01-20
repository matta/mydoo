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
      testDir: testDirDesktop,
      use: { ...devices["Desktop Chrome"] },
      grepInvert: /@migration-pending/,
    },
    {
      name: "bdd-mobile",
      testDir: testDirMobile,
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
