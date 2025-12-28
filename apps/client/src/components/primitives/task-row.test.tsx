import type {Task, TaskID} from '@mydoo/tasklens';
import {TaskStatus} from '@mydoo/tasklens';
import {screen} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {describe, expect, it, vi} from 'vitest';

import {renderWithTestProviders} from '../../test/setup';
import {TaskRow} from './task-row';

function createMockTask(overrides: Partial<Task> = {}): Task {
  return {
    id: '1' as TaskID,
    title: 'Test Task',
    isAcknowledged: false, // Default
    status: TaskStatus.Pending,
    importance: 0.5,
    childTaskIds: [],
    credits: 0,
    creditsTimestamp: Date.now(),
    creditIncrement: 1,
    desiredCredits: 10,
    isSequential: false,
    priorityTimestamp: Date.now(),
    schedule: {
      type: 'Once',
      leadTime: 0,
    },
    notes: '',
    isContainer: false,
    isPending: true,
    isReady: true,
    ...overrides,
  };
}

describe('TaskRow', () => {
  it('renders task title', () => {
    const task = createMockTask({title: 'Buy Groceries'});
    renderWithTestProviders(<TaskRow onToggle={vi.fn()} task={task} />);

    expect(screen.getByText('Buy Groceries')).toBeInTheDocument();
  });

  it('calls onToggle with task id when checkbox is clicked', async () => {
    const user = userEvent.setup();
    const onToggle = vi.fn();
    const task = createMockTask({id: 'task-123' as TaskID});

    renderWithTestProviders(<TaskRow onToggle={onToggle} task={task} />);

    const checkbox = screen.getByRole('checkbox');
    await user.click(checkbox);

    expect(onToggle).toHaveBeenCalledWith('task-123');
  });

  it('shows unchecked checkbox for Pending task', () => {
    const task = createMockTask({status: TaskStatus.Pending});
    renderWithTestProviders(<TaskRow onToggle={vi.fn()} task={task} />);

    const checkbox = screen.getByRole('checkbox');
    expect(checkbox).not.toBeChecked();
  });

  it('shows checked checkbox for Done task', () => {
    const task = createMockTask({status: TaskStatus.Done});
    renderWithTestProviders(<TaskRow onToggle={vi.fn()} task={task} />);

    const checkbox = screen.getByRole('checkbox');
    expect(checkbox).toBeChecked();
  });

  it('displays task importance', () => {
    const task = createMockTask({importance: 0.75});
    renderWithTestProviders(<TaskRow onToggle={vi.fn()} task={task} />);

    expect(screen.getByText('0.75')).toBeInTheDocument();
  });
});
