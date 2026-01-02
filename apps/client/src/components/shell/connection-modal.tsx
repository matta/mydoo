import {
  type AutomergeUrl,
  isValidAutomergeUrl,
} from '@automerge/automerge-repo';
import {
  Button,
  Code,
  Group,
  Modal,
  Stack,
  Text,
  TextInput,
} from '@mantine/core';
import {useState} from 'react';

/**
 * Props for the ConnectionModal component.
 */
interface ConnectionModalProps {
  /** Whether the modal is open. */
  opened: boolean;
  /** Callback fired when the modal requests to close. */
  onClose: () => void;
  /** The current Automerge document URL. */
  currentUrl: AutomergeUrl;
  /** Callback fired when the user requests to reset/create a new document. */
  onReset: () => void;
  /** Callback fired when the user requests to connect to a specific document ID. */
  onConnect: (url: AutomergeUrl) => void;
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
  onReset,
  onConnect,
}: ConnectionModalProps) {
  const [inputValue, setInputValue] = useState('');

  const urlString = inputValue.trim();
  const isValid = isValidAutomergeUrl(urlString);
  const showError = urlString !== '' && !isValid;

  const handleConnect = () => {
    if (isValid) {
      onConnect(urlString as AutomergeUrl);
    }
  };

  return (
    <Modal opened={opened} onClose={onClose} title="Connection Info">
      <Stack>
        <Text size="sm">
          Your data is stored locally. This is your Document ID:
        </Text>

        <Code block data-testid="document-id">
          {currentUrl}
        </Code>

        <Button
          variant="outline"
          color="red"
          onClick={onReset}
          data-testid="reset-document-button"
        >
          Create New Document
        </Button>

        <Text size="sm" mt="md" fw={500}>
          Switch Document
        </Text>
        <Group align="flex-end">
          <TextInput
            placeholder="automerge:..."
            label="Document ID"
            value={inputValue}
            onChange={e => setInputValue(e.currentTarget.value)}
            style={{flex: 1}}
            data-testid="connect-document-input"
            error={showError ? 'Invalid Automerge URI format' : null}
          />
          <Button
            onClick={handleConnect}
            disabled={!isValid}
            data-testid="connect-document-button"
          >
            Connect
          </Button>
        </Group>

        <Button fullWidth onClick={onClose}>
          Close
        </Button>
      </Stack>
    </Modal>
  );
}
