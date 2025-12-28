import {MantineProvider} from '@mantine/core';

import {AppShellContainer} from './components/shell/AppShellContainer';
import {SeedData} from './dev/SeedData';
import {NavigationProvider} from './viewmodel/ui/use-navigation-state';
import {useDocument} from './viewmodel/use-document';

import '@mantine/core/styles.css';

function App() {
  const docUrl = useDocument();

  return (
    <MantineProvider>
      <NavigationProvider>
        {docUrl ? (
          <>
            <SeedData docUrl={docUrl} />
            <AppShellContainer docUrl={docUrl} />
          </>
        ) : (
          // biome-ignore lint/complexity/noUselessFragments: unblocks TS build
          <></>
        )}
      </NavigationProvider>
    </MantineProvider>
  );
}

export default App;
