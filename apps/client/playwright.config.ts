import { defineConfig, devices } from '@playwright/test';
import { defineBddConfig } from 'playwright-bdd';

const isAgent = !!process.env.ANTIGRAVITY_AGENT || !!process.env.GEMINI_CLI;

const bddCommon = {
  features: 'tests/e2e/features/*.feature',
  steps: 'tests/e2e/{steps/*.steps.ts,fixtures.ts}',
};

const testDirDesktop = defineBddConfig({
  ...bddCommon,
  outputDir: 'tests/e2e/.features-gen/desktop',
});

const testDirMobile = defineBddConfig({
  ...bddCommon,
  outputDir: 'tests/e2e/.features-gen/mobile',
});

export default defineConfig({
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  ...(process.env.CI ? { workers: 1 } : {}),
  reporter: isAgent || process.env.CI ? [['html', { open: 'never' }]] : 'html',
  use: {
    baseURL: 'http://localhost:5179',
    trace: 'on-first-retry',
  },
  webServer: {
    command: 'pnpm run dev --port 5179 --strictPort',
    url: 'http://localhost:5179',
    reuseExistingServer: !process.env.CI,
  },
  projects: [
    {
      name: 'bdd-desktop',
      testDir: testDirDesktop,
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'bdd-mobile',
      testDir: testDirMobile,
      use: { ...devices['Pixel 7'] },
      grepInvert: /@skip-mobile/,
    },
    {
      name: 'e2e',
      testDir: 'tests/e2e',
      testIgnore: ['features/**', 'steps/**', '.features-gen*/**'],
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
