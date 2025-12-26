import {
  createTheme,
  MantineProvider,
  Menu,
  Modal,
  Popover,
} from '@mantine/core';
import {
  type RenderOptions,
  type RenderResult,
  render as testingLibraryRender,
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
// We use duration: 1 (not 0) because duration: 0 breaks userEvent interaction
// with Mantine's Menu component - the internal state machine doesn't complete.
// 1ms is effectively instant but allows the state machine to work.
const testingTheme = createTheme({
  components: {
    // Disable Menu transitions for tests
    Menu: Menu.extend({
      defaultProps: {
        transitionProps: {duration: 1},
      },
    }),
    // Disable Modal transitions for tests
    Modal: Modal.extend({
      defaultProps: {
        transitionProps: {duration: 1},
      },
    }),
    // Disable Popover transitions for tests (Menu uses this internally)
    Popover: Popover.extend({
      defaultProps: {
        transitionProps: {duration: 1},
      },
    }),
  },
});

/**
 * Custom render function that wraps components with MantineProvider and custom test theme.
 * Use this instead of @testing-library/react's render for Mantine components.
 */
function AllProviders({children}: PropsWithChildren) {
  return <MantineProvider theme={testingTheme}>{children}</MantineProvider>;
}

export function renderWithTestProviders(
  ui: React.ReactNode,
  options?: Omit<RenderOptions, 'wrapper'>,
): RenderResult {
  return testingLibraryRender(ui, {wrapper: AllProviders, ...options});
}

/**
 * TESTING MANTINE ASYNC COMPONENTS
 *
 * Mantine components (Menu, Modal, etc.) have transitions and async state machines.
 * Tests that interact with these components should use "await" with "findBy*" queries
 * to ensure the component has reached the expected state.
 *
 * PATTERN:
 * const user = userEvent.setup();
 * await user.click(menuTrigger);
 * // PREFERRED: findByRole waits automatically (up to 1000ms) for the item to appear
 * const item = await screen.findByRole('menuitem', { name: /action/i });
 * await user.click(item);
 *
 * // ALTERNATIVE: waitFor (only if findBy* isn't applicable)
 * await waitFor(() => expect(something).toBeVisible());
 */
