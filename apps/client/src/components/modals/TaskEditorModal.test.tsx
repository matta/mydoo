import {MantineProvider} from '@mantine/core';
import type {Task, TaskID} from '@mydoo/tasklens';
import {fireEvent, render, screen} from '@testing-library/react';
import {describe, expect, it, vi} from 'vitest';

import {TaskEditorModal} from './TaskEditorModal';

// Mock @mantine/dates since it requires DatesProvider
vi.mock('@mantine/dates', () => ({
  DateInput: ({label}: {label: string}) => (
    <div data-testid="date-input">{label}</div>
  ),
  DatePickerInput: ({label}: {label: string}) => (
    <div data-testid="date-picker-input">{label}</div>
  ),
}));

const renderWithProviders = (ui: React.ReactElement) =>
  render(<MantineProvider>{ui}</MantineProvider>);

const mockTask: Task = {
  id: 'task-1' as TaskID,
  title: 'Test Task',
  status: 'Pending',
  importance: 0.7,
  creditIncrement: 3,
  credits: 0,
  desiredCredits: 0,
  creditsTimestamp: Date.now(),
  priorityTimestamp: Date.now(),
  schedule: {
    type: 'Once',
    leadTime: 7 * 24 * 60 * 60 * 1000, // 7 days
  },
  isSequential: false,
  childTaskIds: [],
  isAcknowledged: false,
};

describe('TaskEditorModal', () => {
  it('renders task title in input', () => {
    renderWithProviders(
      <TaskEditorModal
        descendantCount={0}
        onAddChild={vi.fn()}
        onAddSibling={vi.fn()}
        onClose={vi.fn()}
        onDelete={vi.fn()}
        onSave={vi.fn()}
        opened={true}
        parentTitle="Parent Task"
        task={mockTask}
      />,
    );

    expect(screen.getByDisplayValue('Test Task')).toBeInTheDocument();
  });

  it('renders parent title', () => {
    renderWithProviders(
      <TaskEditorModal
        descendantCount={0}
        onAddChild={vi.fn()}
        onAddSibling={vi.fn()}
        onClose={vi.fn()}
        onDelete={vi.fn()}
        onSave={vi.fn()}
        opened={true}
        parentTitle="My Parent"
        task={mockTask}
      />,
    );

    expect(screen.getByText(/Parent: My Parent/)).toBeInTheDocument();
  });

  it('shows Root when no parent', () => {
    renderWithProviders(
      <TaskEditorModal
        descendantCount={0}
        onAddChild={vi.fn()}
        onAddSibling={vi.fn()}
        onClose={vi.fn()}
        onDelete={vi.fn()}
        onSave={vi.fn()}
        opened={true}
        parentTitle={null}
        task={mockTask}
      />,
    );

    expect(screen.getByText(/Root \(Top Level\)/)).toBeInTheDocument();
  });

  it('calls onAddSibling when Add Sibling clicked', () => {
    const onAddSibling = vi.fn();
    const taskWithParent = {...mockTask, parentId: 'parent-1' as TaskID};

    renderWithProviders(
      <TaskEditorModal
        descendantCount={0}
        onAddChild={vi.fn()}
        onAddSibling={onAddSibling}
        onClose={vi.fn()}
        onDelete={vi.fn()}
        onSave={vi.fn()}
        opened={true}
        parentTitle="Parent"
        task={taskWithParent}
      />,
    );

    fireEvent.click(screen.getByRole('button', {name: /add sibling/i}));

    expect(onAddSibling).toHaveBeenCalledWith('parent-1');
  });

  it('calls onAddChild when Add Child clicked', () => {
    const onAddChild = vi.fn();

    renderWithProviders(
      <TaskEditorModal
        descendantCount={0}
        onAddChild={onAddChild}
        onAddSibling={vi.fn()}
        onClose={vi.fn()}
        onDelete={vi.fn()}
        onSave={vi.fn()}
        opened={true}
        parentTitle={null}
        task={mockTask}
      />,
    );

    fireEvent.click(screen.getByRole('button', {name: /add child/i}));

    expect(onAddChild).toHaveBeenCalledWith('task-1');
  });

  it('calls onDelete when Delete clicked', () => {
    const onDelete = vi.fn();

    renderWithProviders(
      <TaskEditorModal
        descendantCount={2}
        onAddChild={vi.fn()}
        onAddSibling={vi.fn()}
        onClose={vi.fn()}
        onDelete={onDelete}
        onSave={vi.fn()}
        opened={true}
        parentTitle={null}
        task={mockTask}
      />,
    );

    fireEvent.click(screen.getByRole('button', {name: /delete/i}));

    expect(onDelete).toHaveBeenCalledWith('task-1', true);
  });

  it('shows loading state when in edit mode but task is null', () => {
    renderWithProviders(
      <TaskEditorModal
        descendantCount={0}
        onAddChild={vi.fn()}
        onAddSibling={vi.fn()}
        onClose={vi.fn()}
        onDelete={vi.fn()}
        onSave={vi.fn()}
        opened={true}
        parentTitle={null}
        task={null}
        mode="edit"
      />,
    );

    expect(screen.getByText('Loading task details...')).toBeInTheDocument();
  });
});
