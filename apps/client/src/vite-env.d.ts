/// <reference types="vite/client" />

// CSS module declarations
declare module '*.css' {
  const content: Record<string, string>;
  export default content;
}

// CSS imports as side effects
declare module '@mantine/core/styles.css';
declare module '@mantine/dates/styles.css';

export interface BuildInfo {
  hash: string;
  date: string;
  clean: boolean;
}

declare global {
  var __BUILD_INFO__: BuildInfo;
}
