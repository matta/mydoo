import {type AutomergeUrl, Repo} from '@automerge/automerge-repo';
import {RepoContext} from '@automerge/automerge-repo-react-hooks';
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

// Mock Automerge Repo
const mockRepo = new Repo({network: []});

/**
 * Custom render function that wraps components with MantineProvider and custom test theme.
 * Use this instead of @testing-library/react's render for Mantine components.
 */

import {
  createEmptyTunnelState,
  createStore,
  TaskLensProvider,
} from '@mydoo/tasklens';

const defaultDocHandle = mockRepo.create(createEmptyTunnelState());
const defaultDocUrl = defaultDocHandle.url;

export function createTestWrapper(
  repo: Repo = mockRepo,
  store = createStore(),
  docUrl: AutomergeUrl = defaultDocUrl,
) {
  return function TestWrapper({children}: PropsWithChildren) {
    return (
      <RepoContext.Provider value={repo}>
        <TaskLensProvider docUrl={docUrl} store={store}>
          <MantineProvider theme={testingTheme}>{children}</MantineProvider>
        </TaskLensProvider>
      </RepoContext.Provider>
    );
  };
}

/**
 * Options for custom render.
 * @param repo - Optional Automerge Repo instance to use in the context.
 */
export interface TestRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  repo?: Repo;
  store?: ReturnType<typeof createStore>;
  url?: AutomergeUrl;
}

/**
 * Render a component with all global providers (Mantine, Repo, etc).
 * @param ui - The component to render
 * @param options - Render options including custom Repo
 */
export function renderWithTestProviders(
  ui: React.ReactNode,
  options: TestRenderOptions = {},
): RenderResult {
  const {repo, store, url, ...renderOptions} = options;
  return testingLibraryRender(ui, {
    wrapper: createTestWrapper(repo, store, url),
    ...renderOptions,
  });
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
