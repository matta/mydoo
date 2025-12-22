import {MantineProvider} from '@mantine/core';

import {DoViewContainer} from './viewmodel/containers/DoViewContainer';
import {useDocument} from './viewmodel/useDocument';

import '@mantine/core/styles.css';

function App() {
  const docUrl = useDocument();

  if (!docUrl) {
    return <></>; // Or a loading spinner
  }

  return (
    <MantineProvider>
      <DoViewContainer docUrl={docUrl} />
    </MantineProvider>
  );
}

export default App;
