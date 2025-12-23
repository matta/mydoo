import {MantineProvider} from '@mantine/core';
import {fireEvent, render, screen} from '@testing-library/react';
import {describe, expect, it, vi} from 'vitest';

import {DeleteConfirmModal} from './DeleteConfirmModal';

const renderWithProviders = (ui: React.ReactElement) =>
  render(<MantineProvider>{ui}</MantineProvider>);

describe('DeleteConfirmModal', () => {
  it('renders task title and descendant count', () => {
    renderWithProviders(
      <DeleteConfirmModal
        descendantCount={3}
        onClose={vi.fn()}
        onConfirm={vi.fn()}
        opened={true}
        taskTitle="Test Task"
      />,
    );

    expect(screen.getByText(/Test Task/)).toBeInTheDocument();
    expect(screen.getByText(/3 sub-tasks/)).toBeInTheDocument();
  });

  it('renders singular sub-task for count of 1', () => {
    renderWithProviders(
      <DeleteConfirmModal
        descendantCount={1}
        onClose={vi.fn()}
        onConfirm={vi.fn()}
        opened={true}
        taskTitle="Another Task"
      />,
    );

    expect(screen.getByText(/1 sub-task\?/)).toBeInTheDocument();
  });

  it('does not show descendant count when 0', () => {
    renderWithProviders(
      <DeleteConfirmModal
        descendantCount={0}
        onClose={vi.fn()}
        onConfirm={vi.fn()}
        opened={true}
        taskTitle="Leaf Task"
      />,
    );

    expect(screen.queryByText(/sub-task/)).not.toBeInTheDocument();
  });

  it('calls onConfirm and onClose when Delete clicked', () => {
    const onConfirm = vi.fn();
    const onClose = vi.fn();

    renderWithProviders(
      <DeleteConfirmModal
        descendantCount={0}
        onClose={onClose}
        onConfirm={onConfirm}
        opened={true}
        taskTitle="Task"
      />,
    );

    fireEvent.click(screen.getByRole('button', {name: /delete/i}));

    expect(onConfirm).toHaveBeenCalled();
    expect(onClose).toHaveBeenCalled();
  });

  it('calls onClose when Cancel clicked', () => {
    const onClose = vi.fn();

    renderWithProviders(
      <DeleteConfirmModal
        descendantCount={0}
        onClose={onClose}
        onConfirm={vi.fn()}
        opened={true}
        taskTitle="Task"
      />,
    );

    fireEvent.click(screen.getByRole('button', {name: /cancel/i}));

    expect(onClose).toHaveBeenCalled();
  });
});
