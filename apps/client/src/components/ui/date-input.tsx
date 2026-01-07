import { Input } from '@mantine/core';
import { useCallback } from 'react';

/**
 * Props for the DateInput component.
 */
export interface DateInputProps {
  /** Label displayed above the input */
  label?: React.ReactNode;
  /** Description displayed below the label */
  description?: React.ReactNode;
  /** Error message displayed below the input */
  error?: React.ReactNode;
  /** Whether the field is required */
  required?: boolean;
  /** The selected date value, or null if no date is selected */
  value: Date | null;
  /** Callback when the date changes */
  onChange: (date: Date | null) => void;
  /** Whether the input is disabled */
  disabled?: boolean;
  /** Additional CSS class name */
  className?: string;
  /** Inline styles */
  style?: React.CSSProperties;
  /** Placeholder text (browser support varies) */
  placeholder?: string;
}

/**
 * A date input component that wraps a native `<input type="date" />`
 * with Mantine styling. Uses browser-native date picker UI.
 */
export function DateInput({
  label,
  description,
  error,
  required,
  value,
  onChange,
  disabled,
  className,
  style,
  placeholder,
}: DateInputProps) {
  // Convert Date to YYYY-MM-DD string
  const formatDate = (date: Date | null): string => {
    if (!date) return '';
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
  };

  const inputValue = formatDate(value);

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const val = e.target.value;
      if (!val) {
        onChange(null);
        return;
      }
      // Create date from YYYY-MM-DD string in local time
      const [year, month, day] = val.split('-').map(Number);
      if (year !== undefined && month !== undefined && day !== undefined) {
        const newDate = new Date(year, month - 1, day);
        onChange(newDate);
      }
    },
    [onChange],
  );

  return (
    <Input.Wrapper
      label={label}
      description={description}
      error={error}
      required={required ?? false}
      {...(className ? { className } : {})}
      {...(style ? { style } : {})}
    >
      <Input
        component="input"
        type="date"
        value={inputValue}
        onChange={handleChange}
        disabled={disabled ?? false}
        data-testid="date-input"
        {...(placeholder ? { placeholder } : {})}
      />
    </Input.Wrapper>
  );
}
