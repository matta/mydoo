import {render, screen} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {describe, expect, it} from 'vitest';

import {createTestWrapper} from '../../test/setup';
import {ConnectionModal} from './connection-modal';

describe('ConnectionModal', () => {
  const currentUrl = 'automerge:12345';

  it('should render the current URL', () => {
    const wrapper = createTestWrapper();
    render(
      <ConnectionModal
        opened={true}
        onClose={() => {}}
        currentUrl={currentUrl}
        onReset={() => {}}
      />,
      {wrapper},
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
      />,
      {wrapper},
    );

    const closeButton = screen.getByRole('button', {name: /close/i});
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
      />,
      {wrapper},
    );

    const resetButton = screen.getByRole('button', {
      name: /create new document/i,
    });
    await userEvent.click(resetButton);

    expect(reset).toBe(true);
  });
});
