import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App.tsx";
import { RepoProvider } from "./hooks/RepoProvider";

const rootElement = document.getElementById("root");
if (!rootElement) throw new Error("Failed to find the root element");

createRoot(rootElement).render(
  <StrictMode>
    <RepoProvider>
      <App />
    </RepoProvider>
  </StrictMode>,
);
