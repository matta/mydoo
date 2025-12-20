import { TextInput } from "@mantine/core";
import { useState, useRef, useEffect } from "react";

interface InlineInputProps {
  initialValue: string;
  onSave: (value: string) => void;
  onCancel: () => void;
  placeholder?: string;
}

export function InlineInput({
  initialValue,
  onSave,
  onCancel,
  placeholder = "Enter text...",
}: InlineInputProps) {
  const [value, setValue] = useState(initialValue);
  const inputRef = useRef<HTMLInputElement>(null);

  // Auto-focus when mounted
  useEffect(() => {
    inputRef.current?.focus();
    // setTimeout to ensure focus works after mount and ready
    setTimeout(() => inputRef.current?.select(), 0);
  }, []);

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter") {
      if (value.trim()) {
        onSave(value.trim());
      } else {
        onCancel();
      }
    } else if (e.key === "Escape") {
      onCancel();
    }
  }

  function handleBlur() {
    if (value.trim()) {
      onSave(value.trim());
    } else {
      onCancel();
    }
  }

  return (
    <TextInput
      ref={inputRef}
      variant="unstyled"
      value={value}
      onChange={(e) => {
        setValue(e.currentTarget.value);
      }}
      onKeyDown={handleKeyDown}
      onBlur={handleBlur}
      placeholder={placeholder}
      styles={{ input: { padding: 0, height: "auto", minHeight: 0 } }}
    />
  );
}
