import {Button, Modal, ScrollArea, Stack, Text, ThemeIcon} from '@mantine/core';
import type {TaskID, TunnelNode} from '@mydoo/tasklens';
import {IconArrowRight, IconTarget} from '@tabler/icons-react';

interface MovePickerModalProps {
  opened: boolean;
  onClose: () => void;
  roots: TunnelNode[];
  onSelect: (parentId: TaskID | undefined) => void;
  taskTitle: string;
}

const TargetItem = ({
  node,
  onSelect,
  depth = 0,
}: {
  node: TunnelNode;
  onSelect: (id: TaskID) => void;
  depth?: number;
}) => {
  return (
    <>
      <Button
        variant="subtle"
        fullWidth
        justify="flex-start"
        onClick={() => onSelect(node.id)}
        pl={`calc(var(--mantine-spacing-sm) + ${depth * 20}px)`}
        leftSection={<IconArrowRight size={12} style={{opacity: 0.5}} />}
        color="gray"
      >
        <Text size="sm" truncate>
          {node.title}
        </Text>
      </Button>
      {node.children.map(child => (
        <TargetItem
          key={child.id}
          node={child}
          onSelect={onSelect}
          depth={depth + 1}
        />
      ))}
    </>
  );
};

export function MovePickerModal({
  opened,
  onClose,
  roots,
  onSelect,
  taskTitle,
}: MovePickerModalProps) {
  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={`Move "${taskTitle}"`}
      centered
      size="md"
    >
      <Stack gap="sm">
        <Text size="xs" c="dimmed">
          Select a new parent:
        </Text>

        <ScrollArea.Autosize mah="60vh">
          <Button
            variant="subtle"
            fullWidth
            justify="flex-start"
            onClick={() => onSelect(undefined)}
            color="gray"
            leftSection={
              <ThemeIcon size="sm" variant="light" color="blue">
                <IconTarget size={14} />
              </ThemeIcon>
            }
          >
            <Text size="sm" fw={500}>
              Root (Top Level)
            </Text>
          </Button>

          {roots.map(node => (
            <TargetItem key={node.id} node={node} onSelect={onSelect} />
          ))}
        </ScrollArea.Autosize>
      </Stack>
    </Modal>
  );
}
