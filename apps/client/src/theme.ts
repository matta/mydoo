import {createTheme, type MantineThemeOverride} from '@mantine/core';

/**
 * Mobile Input Zoom Fix
 *
 * iOS Safari automatically zooms in when an input element has a font-size of
 * less than 16px. This creates a frustrating UX where the user has to manually
 * zoom out after entering text. To prevent this, we enforce a minimum font-size
 * of 16px on all input components.
 *
 * Ideally, we would target only mobile devices using media queries, but for
 * simplicity and robustness (ensuring no edge cases with specific component
 * sizes), we apply this globally or rely on the fact that 16px is a reasonable
 * default for readability anyway.
 */
const MIN_IOS_INPUT_FONT_SIZE = '16px';

const inputStyles = {
  styles: {
    input: {
      fontSize: MIN_IOS_INPUT_FONT_SIZE,
    },
  },
};

export const theme: MantineThemeOverride = createTheme({
  components: {
    TextInput: inputStyles,
    NumberInput: inputStyles,
    PasswordInput: inputStyles,
    Select: inputStyles,
    Textarea: inputStyles,
    DatePickerInput: inputStyles,
  },
});
