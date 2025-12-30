import {Button, Code, Modal, Stack, Text} from '@mantine/core';

/**
 * Props for the ConnectionModal component.
 */
interface ConnectionModalProps {
  /** Whether the modal is open. */
  opened: boolean;
  /** Callback fired when the modal requests to close. */
  onClose: () => void;
  /** The current Automerge document URL. */
  currentUrl: string;
}

/**
 * Modal for managing Automerge document connection.
 * Currently just displays the active Document ID for debugging.
 *
 * @param props - Component props.
 * @param props.opened - Whether the modal is open.
 * @param props.onClose - Callback fired when the modal requests to close.
 * @param props.currentUrl - The current Automerge document URL.
 */
export function ConnectionModal({
  opened,
  onClose,
  currentUrl,
}: ConnectionModalProps) {
  return (
    <Modal opened={opened} onClose={onClose} title="Connection Info">
      <Stack>
        <Text size="sm">
          Your data is stored locally. This is your Document ID:
        </Text>

        <Code block>{currentUrl}</Code>

        <Button fullWidth onClick={onClose}>
          Close
        </Button>
      </Stack>
    </Modal>
  );
}
