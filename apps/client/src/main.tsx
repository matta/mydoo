import { MantineProvider } from "@mantine/core";
import { Notifications } from "@mantine/notifications";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import App from "./app";
import { ErrorBoundary } from "./components/error-boundary";
import { ReloadPrompt } from "./components/pwa/reload-prompt";
import { RepoProvider } from "./hooks/repo-provider";
import { theme } from "./theme";

import "@mantine/core/styles.css";
import "@mantine/notifications/styles.css";

const rootElement = document.getElementById("root");
if (!rootElement) throw new Error("Failed to find the root element");

createRoot(rootElement).render(
  <StrictMode>
    <MantineProvider defaultColorScheme="auto" theme={theme}>
      <Notifications position="top-right" />
      <ErrorBoundary>
        <RepoProvider>
          <App />
          <ReloadPrompt />
        </RepoProvider>
      </ErrorBoundary>
    </MantineProvider>
  </StrictMode>,
);
