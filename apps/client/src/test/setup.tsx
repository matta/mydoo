import {createTheme, MantineProvider, Menu, Modal} from '@mantine/core';
import {
  type RenderOptions,
  type RenderResult,
  render as rtlRender,
} from '@testing-library/react';
import type {PropsWithChildren} from 'react';

import '@testing-library/jest-dom/vitest';

// Mock for window.matchMedia - required by Mantine's color scheme detection
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string): MediaQueryList => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => undefined,
    removeListener: () => undefined,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
    dispatchEvent: () => true,
  }),
});

// Mock ResizeObserver - required by Mantine's modal and floating components
class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}
window.ResizeObserver = MockResizeObserver;

// Define global test theme with transitions disabled
// We disable transitions in tests to prevent flakiness caused by timing issues with
// animations (especially for Modals and Menus) and to speed up test execution.
const testTheme = createTheme({
  components: {
    // Disable Menu transitions for tests
    Menu: Menu.extend({
      defaultProps: {
        transitionProps: {duration: 0},
      },
    }),
    // Disable Modal transitions for tests
    Modal: Modal.extend({
      defaultProps: {
        transitionProps: {duration: 0},
      },
    }),
  },
});

/**
 * Custom render function that wraps components with MantineProvider and custom test theme.
 * Use this instead of @testing-library/react's render for Mantine components.
 */
function AllProviders({children}: PropsWithChildren) {
  return <MantineProvider theme={testTheme}>{children}</MantineProvider>;
}

export function customRender(
  ui: React.ReactNode,
  options?: Omit<RenderOptions, 'wrapper'>,
): RenderResult {
  return rtlRender(ui, {wrapper: AllProviders, ...options});
}
