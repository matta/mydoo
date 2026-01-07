/**
 * DeleteConfirmModal: Confirmation dialog before deleting a task with children.
 *
 * Per PRD §3.6, this modal must show the count of descendants that will be deleted.
 * Leaf tasks (no children) may be deleted without this confirmation.
 */
import { Button, Group, Modal, Stack, Text } from '@mantine/core';

interface DeleteConfirmModalProps {
  /** Whether the modal is open */
  opened: boolean;
  /** Callback to close the modal */
  onClose: () => void;
  /** Callback when user confirms deletion */
  onConfirm: () => void;
  /** Title of the task being deleted */
  taskTitle: string;
  /** Number of descendants that will also be deleted */
  descendantCount: number;
}

export function DeleteConfirmModal({
  opened,
  onClose,
  onConfirm,
  taskTitle,
  descendantCount,
}: DeleteConfirmModalProps) {
  const handleConfirm = () => {
    onConfirm();
    onClose();
  };

  return (
    <Modal
      centered
      onClose={onClose}
      opened={opened}
      size="sm"
      title="Delete Task"
    >
      <Stack gap="md">
        <Text>
          Are you sure you want to delete <strong>"{taskTitle}"</strong>
          {descendantCount > 0 &&
            ` and ${String(descendantCount)} sub-task${descendantCount === 1 ? '' : 's'}`}
          ?
        </Text>
        <Text c="dimmed" size="sm">
          This action cannot be undone.
        </Text>
        <Group justify="flex-end">
          <Button onClick={onClose} variant="default">
            Cancel
          </Button>
          <Button color="red" onClick={handleConfirm}>
            Delete
          </Button>
        </Group>
      </Stack>
    </Modal>
  );
}
