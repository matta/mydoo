import { Repo } from '@automerge/automerge-repo';
import { createEmptyTunnelState } from '@mydoo/tasklens';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterAll, beforeAll, describe, expect, it, vi } from 'vitest';
import { renderWithTestProviders } from '../../test/setup';
import { NavigationProvider } from '../../viewmodel/ui/use-navigation-state';
import { AppShellContainer } from './app-shell-container';

describe('AppShellContainer', () => {
  // Mock URL.createObjectURL and URL.revokeObjectURL
  const originalCreateObjectURL = URL.createObjectURL;
  const originalRevokeObjectURL = URL.revokeObjectURL;
  const mockCreateObjectURL = vi.fn();
  const mockRevokeObjectURL = vi.fn();

  beforeAll(() => {
    URL.createObjectURL = mockCreateObjectURL;
    URL.revokeObjectURL = mockRevokeObjectURL;
  });

  afterAll(() => {
    URL.createObjectURL = originalCreateObjectURL;
    URL.revokeObjectURL = originalRevokeObjectURL;
  });

  it('should download the document as JSON when "Download JSON" is clicked', async () => {
    // Setup repo and doc
    const repo = new Repo({ network: [] });
    const handle = repo.create(createEmptyTunnelState());
    const docUrl = handle.url;

    // Mock HTMLAnchorElement.prototype.click
    const clickSpy = vi.spyOn(HTMLAnchorElement.prototype, 'click');
    mockCreateObjectURL.mockReturnValue('blob:mock-url');

    renderWithTestProviders(
      <NavigationProvider>
        <AppShellContainer docUrl={docUrl} />
      </NavigationProvider>,
      {
        repo,
        url: docUrl,
      },
    );

    // Open Options Menu
    const optionsButton = screen.getByRole('button', { name: /options/i });
    await userEvent.click(optionsButton);

    // Find and Click Download JSON
    const downloadButton = await screen.findByRole('menuitem', {
      name: /download json/i,
    });
    await userEvent.click(downloadButton);

    // Assertions
    expect(mockCreateObjectURL).toHaveBeenCalled();
    const mockCall = mockCreateObjectURL.mock.calls[0];
    if (!mockCall) throw new Error('mockCreateObjectURL was not called');
    const blob = mockCall[0] as Blob;
    expect(blob).toBeInstanceOf(Blob);

    // Read the blob content to verify it contains the document
    const text = await blob.text();
    const json = JSON.parse(text);

    // Automerge doc should have the initial state
    expect(json).toHaveProperty('tasks');

    expect(clickSpy).toHaveBeenCalled();
    expect(mockRevokeObjectURL).toHaveBeenCalledWith('blob:mock-url');
  });
});
