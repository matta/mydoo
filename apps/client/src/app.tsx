import { MantineProvider } from "@mantine/core";
import { useMemo } from "react";
import { Provider } from "react-redux";

import { AppShellContainer } from "./components/shell/app-shell-container";
import { SeedData } from "./dev/seed-data";
import { createClientStore } from "./store";
import { NavigationProvider } from "./viewmodel/ui/use-navigation-state";
import { useDocument } from "./viewmodel/use-document";

import "@mantine/core/styles.css";

function App() {
  const docUrl = useDocument();

  // Create Redux store when docUrl is ready.
  // Memoize to prevent recreation on re-renders, but recreate if docUrl changes.
  const store = useMemo(() => {
    if (!docUrl) return null;
    return createClientStore(docUrl);
  }, [docUrl]);

  return (
    <MantineProvider>
      <NavigationProvider>
        {docUrl && store ? (
          <Provider store={store}>
            <SeedData docUrl={docUrl} />
            <AppShellContainer docUrl={docUrl} />
          </Provider>
        ) : (
          // biome-ignore lint/complexity/noUselessFragments: unblocks TS build
          <></>
        )}
      </NavigationProvider>
    </MantineProvider>
  );
}

export default App;
