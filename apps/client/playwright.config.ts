import {defineConfig, devices} from '@playwright/test';
import {defineBddConfig} from 'playwright-bdd';

const isAgent = !!process.env.ANTIGRAVITY_AGENT || !!process.env.GEMINI_CLI;

const PORT = process.env.PLAYWRIGHT_TEST_PORT || '5179';

const testDir = defineBddConfig({
  features: 'tests/e2e/features/*.feature',
  steps: 'tests/e2e/{steps/*.steps.ts,fixtures.ts}',
  outputDir: 'tests/e2e/.features-gen',
});

export default defineConfig({
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  ...(process.env.CI ? {workers: 1} : {}),
  reporter: isAgent || process.env.CI ? [['html', {open: 'never'}]] : 'html',
  use: {
    baseURL: `http://localhost:${PORT}`,
    trace: 'on-first-retry',
  },
  webServer: {
    command: `pnpm run dev --port ${PORT} --strictPort`,
    url: `http://localhost:${PORT}`,
    reuseExistingServer: !process.env.CI,
  },
  projects: [
    {
      name: 'bdd-desktop',
      testDir,
      use: {...devices['Desktop Chrome']},
    },
    {
      name: 'bdd-mobile',
      testDir,
      use: {...devices['Pixel 7']},
    },
    {
      name: 'e2e',
      testDir: 'tests/e2e',
      testIgnore: ['features/**', 'steps/**', '.features-gen/**'],
      use: {...devices['Desktop Chrome']},
    },
  ],
});
