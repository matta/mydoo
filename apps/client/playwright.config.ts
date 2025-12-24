import {defineConfig, devices} from '@playwright/test';

const isAgent = !!process.env.ANTIGRAVITY_AGENT || !!process.env.GEMINI_CLI;

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: isAgent || process.env.CI ? [['html', {open: 'never'}]] : 'html',
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
      name: 'chromium',
      use: {...devices['Desktop Chrome']},
    },
    // Add mobile/firefox later as needed
  ],
});
