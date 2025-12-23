import {MantineProvider} from '@mantine/core';

import {SeedData} from './dev/SeedData';
import {DoViewContainer} from './viewmodel/containers/DoViewContainer';
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
      <SeedData docUrl={docUrl} />
      <DoViewContainer docUrl={docUrl} />
    </MantineProvider>
  );
}

export default App;
