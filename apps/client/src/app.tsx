import type {AnyDocumentId} from '@automerge/automerge-repo';
import {useDocHandle} from '@automerge/automerge-repo-react-hooks';
import {MantineProvider} from '@mantine/core';

import {TaskLensProvider, type TunnelState} from '@mydoo/tasklens';
import {AppShellContainer} from './components/shell/app-shell-container';
import {SeedData} from './dev/seed-data';
import {NavigationProvider} from './viewmodel/ui/use-navigation-state';
import {useDocument} from './viewmodel/use-document';

import '@mantine/core/styles.css';

function App() {
  const docUrl = useDocument();
  const handle = useDocHandle<TunnelState>(docUrl as unknown as AnyDocumentId);

  return (
    <MantineProvider>
      <NavigationProvider>
        {docUrl ? (
          <TaskLensProvider docHandle={handle}>
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
