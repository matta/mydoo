import type { AutomergeUrl } from '@automerge/automerge-repo';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { createTestWrapper } from '../../test/setup';
import { ConnectionModal } from './connection-modal';

describe('ConnectionModal', () => {
  const currentUrl = 'automerge:12345' as AutomergeUrl;

  it('should render the current URL', () => {
    const wrapper = createTestWrapper();
    render(
      <ConnectionModal
        opened={true}
        onClose={() => {}}
        currentUrl={currentUrl}
        onReset={() => {}}
        onConnect={() => {}}
      />,
      { wrapper },
    );

    expect(screen.getByText(currentUrl)).toBeInTheDocument();
  });

  it('should call onClose when close button is clicked', async () => {
    const wrapper = createTestWrapper();
    let closed = false;
    const onClose = () => {
      closed = true;
    };

    render(
      <ConnectionModal
        opened={true}
        onClose={onClose}
        currentUrl={currentUrl}
        onReset={() => {}}
        onConnect={() => {}}
      />,
      { wrapper },
    );

    const closeButton = screen.getByRole('button', { name: /close/i });
    await userEvent.click(closeButton);

    expect(closed).toBe(true);
  });

  it('should call onReset when Create New Document button is clicked', async () => {
    const wrapper = createTestWrapper();
    let reset = false;
    const onReset = () => {
      reset = true;
    };

    render(
      <ConnectionModal
        opened={true}
        onClose={() => {}}
        currentUrl={currentUrl}
        onReset={onReset}
        onConnect={() => {}}
      />,
      { wrapper },
    );

    const resetButton = screen.getByRole('button', {
      name: /create new document/i,
    });
    await userEvent.click(resetButton);

    expect(reset).toBe(true);
  });

  it('should call onConnect when Connect button is clicked', async () => {
    const wrapper = createTestWrapper();
    let connectedUrl = '';
    const onConnect = (url: string) => {
      connectedUrl = url;
    };

    render(
      <ConnectionModal
        opened={true}
        onClose={() => {}}
        currentUrl={currentUrl}
        onReset={() => {}}
        onConnect={onConnect}
      />,
      { wrapper },
    );

    const input = screen.getByLabelText(/document id/i);
    const connectButton = screen.getByRole('button', { name: /connect/i });

    await userEvent.type(input, 'automerge:2zYo9pk9VrPSc5eziZM1337DEvzf');
    await userEvent.click(connectButton);

    expect(connectedUrl).toBe('automerge:2zYo9pk9VrPSc5eziZM1337DEvzf');
  });

  it('should disable Connect button and show error for invalid URL', async () => {
    const wrapper = createTestWrapper();
    render(
      <ConnectionModal
        opened={true}
        onClose={() => {}}
        currentUrl={currentUrl}
        onReset={() => {}}
        onConnect={() => {}}
      />,
      { wrapper },
    );

    const input = screen.getByLabelText(/document id/i);
    const connectButton = screen.getByRole('button', { name: /connect/i });

    // Valid input initially? Input is empty by default, button should be disabled
    expect(connectButton).toBeDisabled();

    // Type invalid URL
    await userEvent.type(input, 'invalid-url');
    expect(connectButton).toBeDisabled();
    expect(
      screen.getByText(/invalid automerge uri format/i),
    ).toBeInTheDocument();

    // Type valid URL
    await userEvent.clear(input);
    await userEvent.type(input, 'automerge:2zYo9pk9VrPSc5eziZM1337DEvzf');
    expect(connectButton).not.toBeDisabled();
    expect(
      screen.queryByText(/invalid automerge uri format/i),
    ).not.toBeInTheDocument();
  });
});
