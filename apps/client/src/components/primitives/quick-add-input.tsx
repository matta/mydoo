import { ActionIcon, TextInput } from '@mantine/core';
import { IconPlus } from '@tabler/icons-react';
import { useState } from 'react';

export interface QuickAddInputProps {
  onAdd: (text: string) => void;
  placeholder?: string;
}

export function QuickAddInput({
  onAdd,
  placeholder = 'Add a new task...',
}: QuickAddInputProps) {
  const [value, setValue] = useState('');

  const handleSubmit = () => {
    const trimmed = value.trim();
    if (trimmed) {
      onAdd(trimmed);
      setValue('');
    }
  };

  return (
    <TextInput
      data-autofocus
      onChange={(event) => {
        setValue(event.currentTarget.value);
      }}
      onKeyDown={(event) => {
        if (event.key === 'Enter') {
          handleSubmit();
        }
      }}
      placeholder={placeholder}
      rightSection={
        <ActionIcon
          aria-label="Add task"
          color="blue"
          disabled={!value.trim()}
          onClick={handleSubmit}
          variant="filled"
        >
          <IconPlus size={16} />
        </ActionIcon>
      }
      size="md"
      value={value}
    />
  );
}
