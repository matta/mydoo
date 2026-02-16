import {
  defineConfig,
  devices,
  type ReporterDescription,
} from "@playwright/test";

const isCI = !!process.env.CI;

/**
 * Build the static file server command lazily so missing env var errors are
 * raised at first use, not when this module is imported. This ensures that
 * IDE tools like knip can load the config without it crashing on a missing env var.
 */
function buildWebServerCommand(): string {
  const webDistDir = process.env.WEB_DIST_DIR;
  if (!webDistDir) {
    return 'echo "Error: WEB_DIST_DIR must be set for Playwright E2E runs. Use \\`just test-e2e*\\` recipes."; exit 1';
  }
  return `pnpm exec serve ${JSON.stringify(webDistDir)} -l tcp://127.0.0.1:5180 -s`;
}

const reporters: ReporterDescription[] = [
  ["html", { open: "never" }],
  ["dot", undefined],
  [
    "junit",
    { outputFile: "test-results/junit.xml", includeProjectInTestName: true },
  ],
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
    get command(): string {
      return buildWebServerCommand();
    },
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
