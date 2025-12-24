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
  /** Callback to delete a task */
  onDelete: (taskId: TaskID, hasChildren: boolean) => void;
  /** Callback to handle creation of a new task */
  onCreate?: (title: string, parentId: TaskID | undefined) => void;
  /** Explicit mode: 'create' or 'edit'. Defaults to inference if not provided (legacy). */
  mode?: 'create' | 'edit' | undefined;
  /** Callback to indent the task */
  onIndent?: (taskId: TaskID) => void;
  /** Callback to outdent the task */
  onOutdent?: (taskId: TaskID) => void;
  /** Whether indentation is possible (has previous sibling) */
  canIndent?: boolean;
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
  onCreate,
  mode,
  onIndent,
  onOutdent,
  canIndent = false,
}: TaskEditorModalProps) {
  // Local form state
  const [title, setTitle] = useState('');
  const [importance, setImportance] = useState(0.5);
  const [effort, setEffort] = useState(0.5);
  const [dueDate, setDueDate] = useState<Date | null>(null);
  const [leadTimeDays, setLeadTimeDays] = useState<number | string>(7);
  const [notes, setNotes] = useState('');

  // Sync form state when task changes (Edit Mode) or clear it (Create Mode)
  useEffect(() => {
    if (task) {
      setTitle(task.title);
      setImportance(task.importance);
      setEffort(task.creditIncrement);
      if (task.schedule.dueDate) {
        setDueDate(new Date(task.schedule.dueDate));
      } else {
        setDueDate(null);
      }
      setLeadTimeDays(Math.round(task.schedule.leadTime / MS_PER_DAY));
      setNotes('');
    } else {
      // Create Mode: Reset to defaults
      setTitle('');
      setImportance(0.5);
      setEffort(0.5);
      setDueDate(null);
      setLeadTimeDays(7);
      setNotes('');
    }
  }, [task, opened]); // Also reset when opening in Create Mode

  const handleSave = useCallback(() => {
    if (task) {
      // Edit Mode
      const dueDateTimestamp = dueDate?.getTime();

      const updates: Partial<Task> = {
        title,
        importance,
        creditIncrement: effort,
        schedule: {
          ...task.schedule,
          dueDate: dueDateTimestamp,
          leadTime: Number(leadTimeDays) * MS_PER_DAY,
        },
      };

      onSave(task.id, updates);
      onClose();
    } else if (onCreate) {
      // Create Mode
      onCreate(title, undefined); // TaskEditorContainer will handle positioning via state
      onClose();
    }
  }, [
    task,
    title,
    importance,
    effort,
    dueDate,
    leadTimeDays,
    onSave,
    onCreate,
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

  // Use explicit mode if provided, otherwise infer from task presence
  const isCreateMode = mode ? mode === 'create' : !task;

  if (mode === 'edit' && !task) {
    // If in edit mode but task is null, show loading state
    return (
      <Modal
        opened={opened}
        onClose={onClose}
        centered
        size="lg"
        title="Loading..."
      >
        <Stack align="center" py="xl">
          <Text>Loading task details...</Text>
        </Stack>
      </Modal>
    );
  }

  return (
    <Modal
      centered
      fullScreen
      onClose={onClose}
      opened={opened}
      size="lg"
      title={isCreateMode ? 'Create Task' : 'Edit Task'}
    >
      <Stack gap="md">
        {/* Title */}
        <TextInput
          label="Title"
          onChange={e => setTitle(e.currentTarget.value)}
          placeholder="What needs to be done?"
          value={title}
          data-autofocus
        />

        {/* Hierarchy Controls (Edit Mode Only) */}
        {!isCreateMode && task && (
          <Stack gap="xs">
            <Text size="sm" fw={500}>
              Hierarchy
            </Text>
            <Group justify="space-between" align="center">
              <Text c="dimmed" size="sm">
                Parent: {parentTitle || 'Root (Top Level)'}
              </Text>
              {/* Move button placeholder (Phase 5 Step 7) */}
            </Group>
            <Group grow>
              <Button
                variant="default"
                onClick={() => onOutdent?.(task.id)}
                disabled={!task.parentId}
                leftSection={<span>←</span>}
              >
                Outdent
              </Button>
              <Button
                variant="default"
                onClick={() => onIndent?.(task.id)}
                rightSection={<span>→</span>}
                disabled={!canIndent}
              >
                Indent
              </Button>
            </Group>
          </Stack>
        )}

        {/* Create Mode Parent Display */}
        {isCreateMode && (
          <Text c="dimmed" size="sm">
            Parent: {parentTitle || 'Root (Top Level)'}
          </Text>
        )}

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
            onChange={value => {
              // DatePickerInput returns string | null in v8.x
              setDueDate(value ? new Date(value) : null);
            }}
            placeholder="Pick a date"
            value={dueDate}
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

        {/* Save/Create Button */}
        <Button fullWidth onClick={handleSave} disabled={!title.trim()}>
          {isCreateMode ? 'Create Task' : 'Save Changes'}
        </Button>

        {/* Footer Actions (Edit Mode Only) */}
        {!isCreateMode && (
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
        )}
      </Stack>
    </Modal>
  );
}
