import type {Task, TaskID} from '@mydoo/tasklens';
import {createMockTask, TaskStatus} from '@mydoo/tasklens';
import {screen} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {describe, expect, it, vi} from 'vitest';

import {renderWithTestProviders} from '../../test/setup';
import {TaskRow} from './task-row';

// Local wrapper to maintain '1' as default ID for these tests
function createLocalMockTask(overrides: Partial<Task> = {}): Task {
  return createMockTask({
    id: '1' as TaskID,
    desiredCredits: 10,
    ...overrides,
  });
}

describe('TaskRow', () => {
  it('renders task title', () => {
    const task = createLocalMockTask({title: 'Buy Groceries'});
    renderWithTestProviders(<TaskRow onToggle={vi.fn()} task={task} />);

    expect(screen.getByText('Buy Groceries')).toBeInTheDocument();
  });

  it('calls onToggle with task id when checkbox is clicked', async () => {
    const user = userEvent.setup();
    const onToggle = vi.fn();
    const task = createLocalMockTask({id: 'task-123' as TaskID});

    renderWithTestProviders(<TaskRow onToggle={onToggle} task={task} />);

    const checkbox = screen.getByRole('checkbox');
    await user.click(checkbox);

    expect(onToggle).toHaveBeenCalledWith('task-123');
  });

  it('shows unchecked checkbox for Pending task', () => {
    const task = createLocalMockTask({status: TaskStatus.Pending});
    renderWithTestProviders(<TaskRow onToggle={vi.fn()} task={task} />);

    const checkbox = screen.getByRole('checkbox');
    expect(checkbox).not.toBeChecked();
  });

  it('shows checked checkbox for Done task', () => {
    const task = createLocalMockTask({status: TaskStatus.Done});
    renderWithTestProviders(<TaskRow onToggle={vi.fn()} task={task} />);

    const checkbox = screen.getByRole('checkbox');
    expect(checkbox).toBeChecked();
  });
});
