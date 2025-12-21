import {StrictMode} from 'react';
import {createRoot} from 'react-dom/client';
import {MantineProvider} from '@mantine/core';
import '@mantine/core/styles.css';

import App from './App';
import {RepoProvider} from './hooks/RepoProvider';

const rootElement = document.getElementById('root');
if (!rootElement) throw new Error('Failed to find the root element');

createRoot(rootElement).render(
  <StrictMode>
    <MantineProvider defaultColorScheme="auto">
      <RepoProvider>
        <App />
      </RepoProvider>
    </MantineProvider>
  </StrictMode>,
);
