import {MantineProvider} from '@mantine/core';

import {AppShellContainer} from './components/shell/AppShellContainer';
import {SeedData} from './dev/SeedData';
import {NavigationProvider} from './viewmodel/ui/useNavigationState';
import {useDocument} from './viewmodel/useDocument';

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
