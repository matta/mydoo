import { render } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { ReloadPrompt } from './reload-prompt';

// --- Mocks ---
// Vitest hoists vi.mock() calls to the top of the file, so these run before
// the component imports. This allows us to intercept module imports.

vi.mock('@mantine/notifications', () => ({
  notifications: {
    show: vi.fn(),
    hide: vi.fn(),
  },
}));

import { useRegisterSW } from 'virtual:pwa-register/react';
// Must import mocked modules AFTER vi.mock() declarations to get the mock.
import { notifications } from '@mantine/notifications';

describe('ReloadPrompt', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('shows notification when new version is available', () => {
    // Setup mock to return needRefresh = true
    vi.mocked(useRegisterSW).mockReturnValue({
      needRefresh: [true, vi.fn()],
      offlineReady: [false, vi.fn()],
      updateServiceWorker: vi.fn(),
    });

    render(<ReloadPrompt />);

    expect(notifications.show).toHaveBeenCalledWith(
      expect.objectContaining({
        title: 'Update Available',
        id: 'sw-update',
      }),
    );
  });

  it('does not show notification when no update is available', () => {
    // Setup mock to return needRefresh = false
    vi.mocked(useRegisterSW).mockReturnValue({
      needRefresh: [false, vi.fn()],
      offlineReady: [false, vi.fn()],
      updateServiceWorker: vi.fn(),
    });

    render(<ReloadPrompt />);

    expect(notifications.show).not.toHaveBeenCalled();
  });
});
