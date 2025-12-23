import {MantineProvider} from '@mantine/core';

import {AppShellContainer} from './components/shell/AppShellContainer';
import {SeedData} from './dev/SeedData';
import {NavigationProvider} from './viewmodel/ui/useNavigationState';
import {useDocument} from './viewmodel/useDocument';

import '@mantine/core/styles.css';

function App() {
  const docUrl = useDocument();

  if (!docUrl) {
    // biome-ignore lint/complexity/noUselessFragments: unblocks TS build
    return <></>; // Or a loading spinner
  }

  return (
    <MantineProvider>
      <NavigationProvider>
        <SeedData docUrl={docUrl} />
        <AppShellContainer docUrl={docUrl} />
      </NavigationProvider>
    </MantineProvider>
  );
}

export default App;
