import {MantineProvider} from '@mantine/core';

import {TaskLensProvider} from '@mydoo/tasklens';
import {AppShellContainer} from './components/shell/app-shell-container';
import {SeedData} from './dev/seed-data';
import {NavigationProvider} from './viewmodel/ui/use-navigation-state';
import {useDocument} from './viewmodel/use-document';

import '@mantine/core/styles.css';

function App() {
  const docUrl = useDocument();

  return (
    <MantineProvider>
      <NavigationProvider>
        {docUrl ? (
          <TaskLensProvider docId={docUrl}>
            <SeedData docUrl={docUrl} />
            <AppShellContainer docUrl={docUrl} />
          </TaskLensProvider>
        ) : (
          // biome-ignore lint/complexity/noUselessFragments: unblocks TS build
          <></>
        )}
      </NavigationProvider>
    </MantineProvider>
  );
}

export default App;
