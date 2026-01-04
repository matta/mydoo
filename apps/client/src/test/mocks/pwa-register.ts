import {vi} from 'vitest';

/**
 * Mock implementation of virtual:pwa-register/react
 *
 * This file is aliased in vitest.config.ts to replace the virtual module
 * provided by vite-plugin-pwa during testing. It mocks the hooks used
 * for service worker registration and updates.
 */

export const useRegisterSW = vi.fn(
  (): {
    needRefresh: [boolean, (value: boolean) => void];
    offlineReady: [boolean, (value: boolean) => void];
    updateServiceWorker: (reloadPage?: boolean) => Promise<void>;
  } => ({
    needRefresh: [false, vi.fn()],
    offlineReady: [false, vi.fn()],
    updateServiceWorker: vi.fn(),
  }),
);
