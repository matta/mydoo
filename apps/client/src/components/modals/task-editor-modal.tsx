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
  Select,
  Slider,
  Stack,
  Text,
  Textarea,
  TextInput,
} from '@mantine/core';
import type {RepeatConfig, Task, TaskID} from '@mydoo/tasklens';
import {useCallback, useEffect, useState} from 'react';
import {DateInput} from '../ui/date-input';

interface TaskEditorModalProps {
  /** Whether the modal is open */
  opened: boolean;
  /** Callback to close the modal */
  onClose: () => void;
  /** The task being edited */
  task: Task | undefined;
  /** Parent task title (read-only display) */
  parentTitle: string | undefined;
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
  onCreate?: (title: string, props?: Partial<Task>) => void;
  /** Explicit mode: 'create' or 'edit'. Defaults to inference if not provided (legacy). */
  mode?: 'create' | 'edit' | undefined;
  /** Callback to indent the task */
  onIndent?: (taskId: TaskID) => void;
  /** Callback to outdent the task */
  onOutdent?: (taskId: TaskID) => void;
  /** Callback to initiate move (reparenting) */
  onMove?: (taskId: TaskID) => void;
  /** Callback to find the task in the plan view */
  onFindInPlan?: (taskId: TaskID) => void;
  /** Whether indentation is possible (has previous sibling) */
  canIndent?: boolean;
}

/** Milliseconds per day for lead time conversion */
const MS_PER_DAY = 1000 * 60 * 60 * 24;
const MS_PER_HOUR = 1000 * 60 * 60;
const MS_PER_MINUTE = 1000 * 60;

/**
 * Converts a lead time in milliseconds to a scalar value and unit.
 */
function parseLeadTime(totalMs: number): {scalar: number; unit: string} {
  if (totalMs % MS_PER_DAY === 0) {
    return {scalar: totalMs / MS_PER_DAY, unit: 'Days'};
  }
  if (totalMs % MS_PER_HOUR === 0) {
    return {scalar: totalMs / MS_PER_HOUR, unit: 'Hours'};
  }
  return {scalar: Math.round(totalMs / MS_PER_MINUTE), unit: 'Minutes'};
}

/**
 * Converts a lead time scalar and unit back to milliseconds.
 */
function leadTimeToMs(scalar: number, unit: string): number {
  switch (unit) {
    case 'Days':
      return scalar * MS_PER_DAY;
    case 'Hours':
      return scalar * MS_PER_HOUR;
    case 'Minutes':
      return scalar * MS_PER_MINUTE;
    default:
      return scalar * MS_PER_DAY;
  }
}

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
  onMove,
  onFindInPlan,
  canIndent = false,
}: TaskEditorModalProps) {
  // Local form state
  const [title, setTitle] = useState('');
  const [importance, setImportance] = useState(0.5);
  const [effort, setEffort] = useState(0.5);
  const [dueDate, setDueDate] = useState<Date | null>(null);

  // Lead Time State
  const [leadTimeScalar, setLeadTimeScalar] = useState<number | string>(7);
  const [leadTimeUnit, setLeadTimeUnit] = useState<string>('Days');

  const [notes, setNotes] = useState('');
  const [frequency, setFrequency] = useState<string | null>(null);
  const [interval, setInterval] = useState<number | string>(1);

  // Sync form state when the modal opens or the task changes
  useEffect(() => {
    if (!opened) return;

    if (task) {
      setTitle(task.title);
      setImportance(task.importance);
      setEffort(task.creditIncrement);
      if (task.schedule.dueDate) {
        setDueDate(new Date(task.schedule.dueDate));
      } else {
        setDueDate(null);
      }

      // Parse Lead Time
      const {scalar, unit} = parseLeadTime(task.schedule.leadTime);
      setLeadTimeScalar(scalar);
      setLeadTimeUnit(unit);

      setNotes(task.notes || '');
      setFrequency(task.repeatConfig?.frequency || null);
      setInterval(task.repeatConfig?.interval || 1);
    } else {
      // Create Mode: Reset to defaults
      setTitle('');
      setImportance(0.5);
      setEffort(0.5);
      setDueDate(null);
      // Default: 7 Days
      setLeadTimeScalar(7);
      setLeadTimeUnit('Days');
      setNotes('');
      setFrequency(null);
      setInterval(1);
    }
  }, [task, opened]); // Sync form state when the modal opens or the task changes

  const handleSave = useCallback(() => {
    const repeatConfig: RepeatConfig | undefined = frequency
      ? {
          frequency: frequency as RepeatConfig['frequency'],
          interval: Number(interval),
        }
      : undefined;

    // Calculate Lead Time in MS
    const leadTimeMs = leadTimeToMs(Number(leadTimeScalar), leadTimeUnit);

    if (task) {
      // Edit Mode
      const dueDateTimestamp = dueDate?.getTime();

      const updates: Partial<Task> = {
        title,
        importance,
        creditIncrement: effort,
        notes,
        repeatConfig,
        schedule: {
          ...task.schedule,
          type: repeatConfig ? 'Routinely' : 'Once',
          dueDate: dueDateTimestamp,
          leadTime: leadTimeMs,
        },
      };

      onSave(task.id, updates);
      onClose();
    } else if (onCreate) {
      // Create Mode
      const newProps: Partial<Task> = {
        importance,
        creditIncrement: effort,
        notes,
        repeatConfig,
        schedule: {
          type: repeatConfig ? 'Routinely' : 'Once',
          dueDate: dueDate?.getTime(),
          leadTime: leadTimeMs,
        },
      };

      onCreate(title, newProps);
      onClose();
    }
  }, [
    task,
    title,
    importance,
    effort,
    notes,
    frequency,
    interval,
    dueDate,
    leadTimeScalar,
    leadTimeUnit,
    onSave,
    onCreate,
    onClose,
  ]);

  const handleAddSibling = useCallback(() => {
    if (!task) return;
    onAddSibling(task.parentId);
    // Note: Do NOT close here - modal transitions to Create mode
  }, [task, onAddSibling]);

  const handleAddChild = useCallback(() => {
    if (!task) return;
    onAddChild(task.id);
    // Note: Do NOT close here - modal transitions to Create mode
  }, [task, onAddChild]);

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
          key={isCreateMode ? 'create' : task?.id}
          label="Title"
          onChange={e => setTitle(e.currentTarget.value)}
          placeholder="What needs to be done?"
          value={title}
          data-autofocus
          autoFocus
          onKeyDown={e => {
            if (e.key === 'Enter') {
              if (title?.trim()) {
                handleSave();
              }
            }
          }}
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
              <Button
                variant="subtle"
                size="xs"
                onClick={() => onMove?.(task.id)}
              >
                Move...
              </Button>
            </Group>
            <Button
              variant="default"
              fullWidth
              leftSection={<span>🎯</span>}
              onClick={() => onFindInPlan?.(task.id)}
            >
              Find in Plan
            </Button>
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
        <Group grow align="flex-end">
          <DateInput
            label="Due Date"
            onChange={setDueDate}
            placeholder="Pick a date"
            value={dueDate}
          />
          <Group gap="xs" style={{flexGrow: 1}}>
            <NumberInput
              id="lead-time-scalar-input"
              label="Lead Time"
              min={0}
              onChange={setLeadTimeScalar}
              value={leadTimeScalar}
              style={{flex: 1}}
            />
            <Select
              id="lead-time-unit-select"
              label="Unit"
              data={['Minutes', 'Hours', 'Days']}
              value={leadTimeUnit}
              onChange={val => val && setLeadTimeUnit(val)}
              style={{width: '100px'}}
              allowDeselect={false}
              comboboxProps={{
                // Force the dropdown to render inside the Modal to avoid
                // FocusTrap conflicts (wedging) caused by Portals in
                // full-screen mode on mobile.
                withinPortal: false,
              }}
            />
          </Group>
        </Group>

        {/* Repetition */}
        <Group grow>
          <Select
            id="repetition-frequency-select"
            label="Repetition"
            placeholder="None"
            data={[
              {value: 'minutes', label: 'Minutes'},
              {value: 'hours', label: 'Hours'},
              {value: 'daily', label: 'Daily'},
              {value: 'weekly', label: 'Weekly'},
              {value: 'monthly', label: 'Monthly'},
              {value: 'yearly', label: 'Yearly'},
            ]}
            value={frequency}
            onChange={setFrequency}
            clearable
            comboboxProps={{
              // Force the dropdown to render inside the Modal to avoid
              // FocusTrap conflicts (wedging) caused by Portals in full-screen
              // mode on mobile.
              withinPortal: false,
            }}
          />
          {frequency && (
            <NumberInput
              id="repetition-interval-input"
              label="Every X units"
              min={1}
              value={interval}
              onChange={setInterval}
            />
          )}
        </Group>

        {/* Notes */}
        <Textarea
          id="task-notes-textarea"
          autosize
          label="Notes"
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
