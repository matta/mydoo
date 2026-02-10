import {
  defineConfig,
  devices,
  type ReporterDescription,
} from "@playwright/test";

const isCI = !!process.env.CI;

const reporters: ReporterDescription[] = [
  ["html", { open: "never" }],
  ["list", undefined],
  ["junit", { outputFile: "junit.xml" }],
];

if (process.env.SHOW_STEPS) {
  reporters.push(["./tests/e2e/reporters/step-reporter.ts", {}]);
}

export default defineConfig({
  workers: 1,
  retries: 0,
  reporter: reporters,
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
      name: "e2e-desktop",
      testDir: "tests/e2e",
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "e2e-mobile",
      testDir: "tests/e2e",
      use: { ...devices["Pixel 7"] },
    },
  ],
});
