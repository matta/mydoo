import type {Task, TaskID} from '@mydoo/tasklens';
import {screen} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {describe, expect, it, vi} from 'vitest';
import {renderWithTestProviders} from '../../test/setup';

import {TaskEditorModal} from './TaskEditorModal';

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
    renderWithTestProviders(
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
    renderWithTestProviders(
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
    renderWithTestProviders(
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

  it('calls onAddSibling when Add Sibling clicked', async () => {
    const onAddSibling = vi.fn();
    const taskWithParent = {...mockTask, parentId: 'parent-1' as TaskID};

    renderWithTestProviders(
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

    await userEvent.click(screen.getByRole('button', {name: /add sibling/i}));

    expect(onAddSibling).toHaveBeenCalledWith('parent-1');
  });

  it('calls onAddChild when Add Child clicked', async () => {
    const onAddChild = vi.fn();

    renderWithTestProviders(
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

    await userEvent.click(screen.getByRole('button', {name: /add child/i}));

    expect(onAddChild).toHaveBeenCalledWith('task-1');
  });

  it('calls onDelete when Delete clicked', async () => {
    const onDelete = vi.fn();

    renderWithTestProviders(
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

    await userEvent.click(screen.getByRole('button', {name: /delete/i}));

    expect(onDelete).toHaveBeenCalledWith('task-1', true);
  });

  it('shows loading state when in edit mode but task is null', () => {
    renderWithTestProviders(
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

  it('disables Indent button when canIndent is false', () => {
    renderWithTestProviders(
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
        canIndent={false}
      />,
    );

    const indentBtn = screen.getByRole('button', {name: /indent/i});
    expect(indentBtn).toBeDisabled();
  });

  it('enables Indent button when canIndent is true', () => {
    renderWithTestProviders(
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
        canIndent={true}
      />,
    );

    const indentBtn = screen.getByRole('button', {name: /indent/i});
    expect(indentBtn).not.toBeDisabled();
  });
});
