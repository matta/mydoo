import path from 'node:path';
import react from '@vitejs/plugin-react';
import { playwright } from '@vitest/browser-playwright';
import topLevelAwait from 'vite-plugin-top-level-await';
import wasm from 'vite-plugin-wasm';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [react(), wasm(), topLevelAwait()],
  define: {
    __BUILD_INFO__: JSON.stringify({
      hash: 'test-hash',
      date: new Date().toISOString(),
      clean: true,
    }),
  },
  resolve: {
    alias: {
      // Mock the PWA virtual module.
      // vite-plugin-pwa uses virtual modules (e.g. virtual:pwa-register/react)
      // which don't exist on disk. For tests, we must alias this to a real file.
      'virtual:pwa-register/react': path.resolve(
        __dirname,
        './src/test/mocks/pwa-register.ts',
      ),
    },
  },
  test: {
    browser: {
      provider: playwright(),
      enabled: true,
      instances: [{ browser: 'chromium' }],
      headless: true,
    },
    globals: true,
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    setupFiles: './src/test/setup.tsx',
  },
});
