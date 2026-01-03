import {createTheme, type MantineThemeOverride, rem} from '@mantine/core';

export const theme: MantineThemeOverride = createTheme({
  /**
   * Mobile Density & Zoom Fix
   *
   * We shift the font scale up compared to Mantine defaults.
   * - default `sm` becomes 16px (was 11px).
   *
   * This achieves two goals:
   * 1. Prevents iOS Safari from zooming in on inputs (requires >= 16px).
   * 2. Creates a less dense, more touch-friendly UI for mobile users.
   *
   * Mantine defaults: xs=10, sm=11, md=14, lg=16, xl=20
   */
  fontSizes: {
    xs: rem(12), // was 10px
    sm: rem(16), // was 11px (Critical: iOS zoom fix)
    md: rem(18), // was 14px
    lg: rem(20), // was 16px
    xl: rem(24), // was 20px
  },

  /**
   * Spacing scale shifted up for touch-friendliness.
   *
   * Mantine defaults: xs=10, sm=12, md=16, lg=20, xl=32
   * We bump each by ~20% to give more breathing room on mobile.
   */
  spacing: {
    xs: rem(12), // was 10px
    sm: rem(14), // was 12px
    md: rem(20), // was 16px
    lg: rem(24), // was 20px
    xl: rem(40), // was 32px
  },
});
