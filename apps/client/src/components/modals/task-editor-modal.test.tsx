import type { Task, TaskID } from '@mydoo/tasklens';
import { createMockTask } from '@mydoo/tasklens';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { renderWithTestProviders } from '../../test/setup';

import { TaskEditorModal } from './task-editor-modal';

const mockTask: Task = createMockTask({
  id: 'task-1' as TaskID,
  title: 'Test Task',
  importance: 0.7,
  creditIncrement: 3,
  schedule: {
    type: 'Once',
    leadTime: 7 * 24 * 60 * 60 * 1000, // 7 days
  },
});

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
        parentTitle={undefined}
        task={mockTask}
      />,
    );

    expect(screen.getByText(/Root \(Top Level\)/)).toBeInTheDocument();
  });

  it('calls onAddSibling when Add Sibling clicked', async () => {
    const user = userEvent.setup();
    const onAddSibling = vi.fn();
    const taskWithParent = { ...mockTask, parentId: 'parent-1' as TaskID };

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

    await user.click(screen.getByRole('button', { name: /add sibling/i }));

    expect(onAddSibling).toHaveBeenCalledWith('parent-1');
  });

  it('calls onAddChild when Add Child clicked', async () => {
    const user = userEvent.setup();
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
        parentTitle={undefined}
        task={mockTask}
      />,
    );

    await user.click(screen.getByRole('button', { name: /add child/i }));

    expect(onAddChild).toHaveBeenCalledWith('task-1');
  });

  it('calls onDelete when Delete clicked', async () => {
    const user = userEvent.setup();
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
        parentTitle={undefined}
        task={mockTask}
      />,
    );

    await user.click(screen.getByRole('button', { name: /delete/i }));

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
        parentTitle={undefined}
        task={undefined}
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
        parentTitle={undefined}
        task={mockTask}
        canIndent={false}
      />,
    );

    const indentBtn = screen.getByRole('button', { name: /indent/i });
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
        parentTitle={undefined}
        task={mockTask}
        canIndent={true}
      />,
    );

    const indentBtn = screen.getByRole('button', { name: /indent/i });
    expect(indentBtn).not.toBeDisabled();
  });

  it('saves notes when changed', async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    renderWithTestProviders(
      <TaskEditorModal
        descendantCount={0}
        onAddChild={vi.fn()}
        onAddSibling={vi.fn()}
        onClose={vi.fn()}
        onDelete={vi.fn()}
        onSave={onSave}
        opened={true}
        parentTitle={undefined}
        task={mockTask}
      />,
    );

    const notesInput = await screen.findByLabelText(/notes/i);
    await user.clear(notesInput);
    await user.type(notesInput, 'New specific notes');

    await user.click(screen.getByRole('button', { name: /save changes/i }));

    expect(onSave).toHaveBeenCalledWith(
      mockTask.id,
      expect.objectContaining({ notes: 'New specific notes' }),
    );
  });

  it('saves repetition config when frequency and interval changed', async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    renderWithTestProviders(
      <TaskEditorModal
        descendantCount={0}
        onAddChild={vi.fn()}
        onAddSibling={vi.fn()}
        onClose={vi.fn()}
        onDelete={vi.fn()}
        onSave={onSave}
        opened={true}
        parentTitle={undefined}
        task={mockTask}
      />,
    );

    // Use ID for stability in Browser Mode
    const freqSelect = document.getElementById('repetition-frequency-select');
    expect(freqSelect).toBeInTheDocument();
    if (!freqSelect) throw new Error('Frequency select not found');

    // Click to open dropdown
    await user.click(freqSelect);

    // In Browser mode, we wait for the option to appear.
    // Mantine animations can take a moment, so we allow a generous timeout.
    const monthlyOption = await screen.findByRole(
      'option',
      { name: /monthly/i },
      { timeout: 3000 },
    );
    await user.click(monthlyOption);

    // Ensure the popover closes before interacting with other elements
    await waitFor(
      () => expect(screen.queryByRole('option')).not.toBeInTheDocument(),
      { timeout: 3000 },
    );

    const intervalInput = document.getElementById('repetition-interval-input');
    expect(intervalInput).toBeInTheDocument();
    if (!intervalInput) throw new Error('Interval input not found');

    await user.clear(intervalInput);
    await user.type(intervalInput, '3');

    await user.click(screen.getByRole('button', { name: /save changes/i }));

    expect(onSave).toHaveBeenCalledWith(
      mockTask.id,
      expect.objectContaining({
        repeatConfig: { frequency: 'monthly', interval: 3 },
        schedule: expect.objectContaining({ type: 'Routinely' }),
      }),
    );
  });

  // TODO: Fix this test in Vitest Browser Mode. The clear button click is not registering.
  // This is covered by E2E tests.
  it.skip('removes repetition config when frequency cleared', async () => {
    const onSave = vi.fn();
    const taskWithRepeat: Task = {
      ...mockTask,
      repeatConfig: { frequency: 'daily', interval: 1 },
      schedule: { ...mockTask.schedule, type: 'Routinely' },
    };

    renderWithTestProviders(
      <TaskEditorModal
        descendantCount={0}
        onAddChild={vi.fn()}
        onAddSibling={vi.fn()}
        onClose={vi.fn()}
        onDelete={vi.fn()}
        onSave={onSave}
        opened={true}
        parentTitle={undefined}
        task={taskWithRepeat}
      />,
    );

    // Test skipped due to clear button interaction issues in browser environment
  });

  it('resets form state when reopened in Create mode', async () => {
    const user = userEvent.setup();
    const { rerender } = renderWithTestProviders(
      <TaskEditorModal
        descendantCount={0}
        onAddChild={vi.fn()}
        onAddSibling={vi.fn()}
        onClose={vi.fn()}
        onDelete={vi.fn()}
        onSave={vi.fn()}
        opened={true}
        parentTitle={undefined}
        task={undefined} // Create mode
      />,
    );

    const titleInput = screen.getByPlaceholderText('What needs to be done?');
    await user.type(titleInput, 'Draft Title');

    // Close the modal
    rerender(
      <TaskEditorModal
        descendantCount={0}
        onAddChild={vi.fn()}
        onAddSibling={vi.fn()}
        onClose={vi.fn()}
        onDelete={vi.fn()}
        onSave={vi.fn()}
        opened={false} // Closed
        parentTitle={undefined}
        task={undefined}
      />,
    );

    // Reopen the modal in Create mode
    rerender(
      <TaskEditorModal
        descendantCount={0}
        onAddChild={vi.fn()}
        onAddSibling={vi.fn()}
        onClose={vi.fn()}
        onDelete={vi.fn()}
        onSave={vi.fn()}
        opened={true} // Reopened
        parentTitle={undefined}
        task={undefined}
      />,
    );

    // Expect title to be empty again
    expect(titleInput).toHaveValue('');
  });
});
