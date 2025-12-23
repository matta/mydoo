import {MantineProvider} from '@mantine/core';
import {type RenderOptions, render as rtlRender} from '@testing-library/react';
import type {PropsWithChildren} from 'react';

import '@testing-library/jest-dom/vitest';

// Mock for window.matchMedia - required by Mantine's color scheme detection
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string): MediaQueryList => ({
    matches: false,
    media: query,
    // eslint-disable-next-line unicorn/no-null -- MediaQueryList.onchange is typed as null
    onchange: null,
    addListener: () => undefined,
    removeListener: () => undefined,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
    dispatchEvent: () => true,
  }),
});

/**
 * Custom render function that wraps components with MantineProvider.
 * Use this instead of @testing-library/react's render for Mantine components.
 */
function AllProviders({children}: PropsWithChildren) {
  return <MantineProvider>{children}</MantineProvider>;
}

export function render(
  ui: React.ReactNode,
  options?: Omit<RenderOptions, 'wrapper'>,
) {
  return rtlRender(ui, {wrapper: AllProviders, ...options});
}
