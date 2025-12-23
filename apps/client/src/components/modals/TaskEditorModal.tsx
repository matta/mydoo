/**
 * TaskEditorModal: Full-screen (mobile) / centered popup (desktop) for editing task details.
 *
 * Per PRD §4.5, this modal provides:
 * - Title input
 * - Parent display (read-only) + "Move..." button (deferred)
 * - Importance slider (0.0 - 1.0)
 * - Effort slider (0.0 - 1.0) → maps directly to creditIncrement
 * - Due Date picker, Lead Time, Repeat selector
 * - Place dropdown
 * - Notes textarea
 * - Footer: Add Sibling, Add Child, Delete
 */
import {
  Button,
  Group,
  Modal,
  NumberInput,
  Slider,
  Stack,
  Text,
  Textarea,
  TextInput,
} from '@mantine/core';
import {DatePickerInput} from '@mantine/dates';
import type {Task, TaskID} from '@mydoo/tasklens';
import {useCallback, useEffect, useState} from 'react';

interface TaskEditorModalProps {
  /** Whether the modal is open */
  opened: boolean;
  /** Callback to close the modal */
  onClose: () => void;
  /** The task being edited */
  task: Task | null;
  /** Parent task title (read-only display) */
  parentTitle: string | null;
  /** Descendant count (for delete confirmation) */
  descendantCount: number;
  /** Callback to save changes */
  onSave: (taskId: TaskID, updates: Partial<Task>) => void;
  /** Callback to add a sibling task */
  onAddSibling: (parentId: TaskID | undefined) => void;
  /** Callback to add a child task */
  onAddChild: (parentId: TaskID) => void;
  onDelete: (taskId: TaskID, hasChildren: boolean) => void;
}

/** Milliseconds per day for lead time conversion */
const MS_PER_DAY = 1000 * 60 * 60 * 24;

export function TaskEditorModal({
  opened,
  onClose,
  task,
  parentTitle,
  descendantCount,
  onSave,
  onAddSibling,
  onAddChild,
  onDelete,
}: TaskEditorModalProps) {
  // Local form state
  const [title, setTitle] = useState('');
  const [importance, setImportance] = useState(0.5);
  const [effort, setEffort] = useState(0.5);
  const [dueDateStr, setDueDateStr] = useState<string | null>(null);
  const [leadTimeDays, setLeadTimeDays] = useState<number | string>(7);
  const [notes, setNotes] = useState('');

  // Sync form state when task changes
  useEffect(() => {
    if (task) {
      setTitle(task.title);
      setImportance(task.importance);
      // creditIncrement is already [0..1]
      setEffort(task.creditIncrement);
      // Convert timestamp to ISO date string for DateInput
      if (task.schedule.dueDate) {
        const dateStr = new Date(task.schedule.dueDate)
          .toISOString()
          .split('T')[0];
        setDueDateStr(dateStr ?? null);
      } else {
        setDueDateStr(null);
      }
      setLeadTimeDays(Math.round(task.schedule.leadTime / MS_PER_DAY));
      // Notes field doesn't exist in current schema - placeholder for future
      setNotes('');
    }
  }, [task]);

  const handleSave = useCallback(() => {
    if (!task) return;

    // Convert date string back to timestamp
    const dueDateTimestamp = dueDateStr
      ? new Date(dueDateStr).getTime()
      : undefined;

    const updates: Partial<Task> = {
      title,
      importance,
      // creditIncrement is [0..1], same as effort slider
      creditIncrement: effort,
      schedule: {
        ...task.schedule,
        dueDate: dueDateTimestamp,
        leadTime: Number(leadTimeDays) * MS_PER_DAY,
      },
    };

    onSave(task.id, updates);
    onClose();
  }, [
    task,
    title,
    importance,
    effort,
    dueDateStr,
    leadTimeDays,
    onSave,
    onClose,
  ]);

  const handleAddSibling = useCallback(() => {
    if (!task) return;
    onAddSibling(task.parentId);
    onClose();
  }, [task, onAddSibling, onClose]);

  const handleAddChild = useCallback(() => {
    if (!task) return;
    onAddChild(task.id);
    onClose();
  }, [task, onAddChild, onClose]);

  const handleDelete = useCallback(() => {
    if (!task) return;
    onDelete(task.id, descendantCount > 0);
    // Don't close here - the parent handles the delete confirmation flow
  }, [task, descendantCount, onDelete]);

  if (!task) return null;

  return (
    <Modal
      centered
      fullScreen
      onClose={onClose}
      opened={opened}
      size="lg"
      title="Edit Task"
    >
      <Stack gap="md">
        {/* Title */}
        <TextInput
          label="Title"
          onChange={e => setTitle(e.currentTarget.value)}
          placeholder="What needs to be done?"
          value={title}
        />

        {/* Parent (read-only) */}
        <Group>
          <Text c="dimmed" size="sm">
            Parent: {parentTitle || 'Root (Top Level)'}
          </Text>
          {/* Move button deferred to Phase 5 */}
        </Group>

        {/* Importance Slider */}
        <Stack gap="xs">
          <Text size="sm">Importance: {importance.toFixed(2)}</Text>
          <Slider
            label={v => v.toFixed(2)}
            marks={[
              {value: 0, label: '0'},
              {value: 0.5, label: '0.5'},
              {value: 1, label: '1'},
            ]}
            max={1}
            min={0}
            onChange={setImportance}
            step={0.01}
            value={importance}
          />
        </Stack>

        {/* Effort Slider */}
        <Stack gap="xs">
          <Text size="sm">Effort: {effort.toFixed(2)}</Text>
          <Slider
            label={v => v.toFixed(2)}
            marks={[
              {value: 0, label: '0'},
              {value: 0.5, label: '0.5'},
              {value: 1, label: '1'},
            ]}
            max={1}
            min={0}
            onChange={setEffort}
            step={0.01}
            value={effort}
          />
        </Stack>

        {/* Scheduling */}
        <Group grow>
          <DatePickerInput
            clearable
            label="Due Date"
            onChange={value => setDueDateStr(value)}
            placeholder="Pick a date"
            value={dueDateStr}
            valueFormat="YYYY-MM-DD"
          />
          <NumberInput
            label="Lead Time (days)"
            min={0}
            onChange={setLeadTimeDays}
            value={leadTimeDays}
          />
        </Group>

        {/* Notes (placeholder - schema doesn't have notes yet) */}
        <Textarea
          autosize
          disabled
          label="Notes (Coming Soon)"
          minRows={3}
          onChange={e => setNotes(e.currentTarget.value)}
          placeholder="Additional details..."
          value={notes}
        />

        {/* Save Button */}
        <Button fullWidth onClick={handleSave}>
          Save Changes
        </Button>

        {/* Footer Actions */}
        <Group grow>
          <Button onClick={handleAddSibling} variant="outline">
            Add Sibling
          </Button>
          <Button onClick={handleAddChild} variant="outline">
            Add Child
          </Button>
          <Button color="red" onClick={handleDelete} variant="outline">
            Delete
          </Button>
        </Group>
      </Stack>
    </Modal>
  );
}
