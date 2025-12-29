/**
 * @file ErrorBoundary.tsx
 * @description Provides a top-level React Error Boundary to catch uncaught runtime exceptions
 * and prevent the "White Screen of Death". It allows the user to view technical debug
 * information and recover the application state.
 */

import {Box, Button, Code, Container, Stack, Text, Title} from '@mantine/core';
import {Component, type ErrorInfo, type ReactNode} from 'react';

interface Props {
  /** The component tree to be protected by the boundary. */
  children: ReactNode;
}

interface State {
  /** The error object captured by the boundary, if any. */
  error: Error | null;
  /** React-specific error metadata, including the component stack trace. */
  errorInfo: ErrorInfo | null;
}

/**
 * ErrorBoundary Component
 *
 * This component acts as a safety net for the application. Initialization
 * failures (like invalid Automerge URLs in the hash) can cause the entire React
 * tree to unmount.
 *
 * KEY RESPONSIBILITIES:
 * 1. Catching errors in its child component tree (lifecycle methods,
 *    rendering, etc.)
 * 2. Logging the failure to the console for developers.
 * 3. Displaying a themed Mantine fallback UI instead of crashing.
 * 4. Providing a recovery path: The "Reset App" button clears the window hash
 *    (the likely source of URL-based crashes) and reloads the application.
 *
 * POSITIONING: It is placed inside `MantineProvider` in `main.tsx` so that the
 * error UI itself has access to the application's design system and theme
 * tokens.
 */
export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {error: null, errorInfo: null};
  }

  override componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Catch errors in any components below and re-render with error message
    this.setState({
      error: error,
      errorInfo: errorInfo,
    });
    // Log error to console for debugging
    console.error('Uncaught error:', error, errorInfo);
  }

  handleReset = () => {
    // Clear hash and reload the app to recover
    //
    // TODO: Replace with a more graceful recovery strategy. For example, we can clear the IndexDB,
    //       remove any service worker state, etc.
    window.location.hash = '';
    window.location.reload();
  };

  override render() {
    if (this.state.error) {
      // Error path
      return (
        <Box
          style={{
            height: '100vh',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: 'var(--mantine-color-gray-0)',
          }}
        >
          <Container size="md">
            <Stack gap="xl">
              <Box>
                <Title order={1} c="red.7">
                  Application Error
                </Title>
                <Text size="lg" mt="md" c="dimmed">
                  Something went wrong. This is likely due to an invalid URL or
                  a temporary data issue.
                </Text>
              </Box>

              <Box>
                <Text fw={700} mb="xs">
                  Error Message
                </Text>
                <Code block color="red.1" c="red.9" p="md">
                  {this.state.error.toString()}
                </Code>
              </Box>

              <Box>
                <Text fw={700} mb="xs">
                  Debug Information
                </Text>
                <Code
                  block
                  style={{
                    maxHeight: '300px',
                    overflow: 'auto',
                    fontSize: '12px',
                  }}
                  p="md"
                >
                  {this.state.errorInfo?.componentStack}
                </Code>
              </Box>

              <Stack gap="sm">
                <Button
                  color="blue"
                  size="md"
                  onClick={this.handleReset}
                  variant="filled"
                >
                  Reset App & New Document
                </Button>
                <Text size="xs" c="dimmed" ta="center">
                  Reporting errors with full debug info to console...
                </Text>
              </Stack>
            </Stack>
          </Container>
        </Box>
      );
    }

    return this.props.children;
  }
}
