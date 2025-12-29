import {MantineProvider} from '@mantine/core';
import {StrictMode} from 'react';
import {createRoot} from 'react-dom/client';

import App from './app';
import {ErrorBoundary} from './components/error-boundary';
import {RepoProvider} from './hooks/repo-provider';

import '@mantine/core/styles.css';
import '@mantine/dates/styles.css';

const rootElement = document.getElementById('root');
if (!rootElement) throw new Error('Failed to find the root element');

createRoot(rootElement).render(
  <StrictMode>
    <MantineProvider defaultColorScheme="auto">
      <ErrorBoundary>
        <RepoProvider>
          <App />
        </RepoProvider>
      </ErrorBoundary>
    </MantineProvider>
  </StrictMode>,
);
