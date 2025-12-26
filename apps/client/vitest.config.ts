import react from '@vitejs/plugin-react';
import {playwright} from '@vitest/browser-playwright';
import topLevelAwait from 'vite-plugin-top-level-await';
import wasm from 'vite-plugin-wasm';
import {defineConfig} from 'vitest/config';

export default defineConfig({
  plugins: [react(), wasm(), topLevelAwait()],
  test: {
    browser: {
      provider: playwright(),
      enabled: true,
      instances: [{browser: 'chromium'}],
      headless: true,
    },
    globals: true,
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    setupFiles: './src/test/setup.tsx',
  },
});
