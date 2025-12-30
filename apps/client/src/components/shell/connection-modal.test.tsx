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
      />,
      {wrapper},
    );

    const closeButton = screen.getByRole('button', {name: /close/i});
    await userEvent.click(closeButton);

    expect(closed).toBe(true);
  });
});
