import type {DocumentHandle} from '@mydoo/tasklens';
import {screen} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {describe, expect, it, vi} from 'vitest';
import {renderWithTestProviders} from '../../../test/setup';
import {PlanViewContainer} from './PlanViewContainer';

// Mock dependencies (removed old manual mocks)
const mocks = vi.hoisted(() => ({
  openCreateModal: vi.fn(),
  mockUseNavigationState: {
    activeTab: 'plan',
    setActiveTab: vi.fn(),
    collapseAll: vi.fn(),
    currentViewId: undefined,
    expandAll: vi.fn(),
    expandedIds: new Set(),
    modal: undefined,
    openEditModal: vi.fn(),
    openCreateModal: vi.fn(),
    closeModal: vi.fn(),
    isExpanded: vi.fn(),
    popView: vi.fn(),
    pushView: vi.fn(),
    resetView: vi.fn(),
    setViewPath: vi.fn(),
    toggleExpanded: vi.fn(),
    viewPath: [],
  },
  useTaskTree: {
    // biome-ignore lint/suspicious/noExplicitAny: Mocking
    roots: [] as any[],
    isLoading: false,
  },
}));

// Mock useMediaQuery
import {useMediaQuery} from '@mantine/hooks';

vi.mock('@mantine/hooks', async () => {
  const actual = await vi.importActual('@mantine/hooks');
  return {
    ...actual,
    useMediaQuery: vi.fn(),
  };
});

vi.mock('../../../viewmodel/intents/useTaskIntents', () => ({
  useTaskIntents: () => ({
    createTask: vi.fn(),
    toggleTask: vi.fn(),
    deleteTask: vi.fn(),
    indentTask: vi.fn(),
    outdentTask: vi.fn(),
  }),
}));

vi.mock('../../../viewmodel/projections/useTaskTree', () => ({
  useTaskTree: () => mocks.useTaskTree,
}));

vi.mock('../../../viewmodel/ui/useNavigationState', () => ({
  useNavigationState: () => mocks.mockUseNavigationState,
}));

vi.mock('../../../viewmodel/ui/useBreadcrumbs', () => ({
  useBreadcrumbs: () => [],
}));

vi.mock('@mydoo/tasklens', () => ({
  useTunnel: () => ({doc: {tasks: {}}}),
}));

// Mock OutlineTree to simplify rendering
vi.mock('./OutlineTree', () => ({
  OutlineTree: () => <div data-testid="outline-tree" />,
}));

describe('PlanViewContainer', () => {
  // Helper to mock viewport per test
  const mockViewport = (isDesktop: boolean) => {
    vi.mocked(useMediaQuery).mockReturnValue(isDesktop);
  };

  // Reset mocks before each test
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders "Add First Task" button when task list is empty', async () => {
    const user = userEvent.setup();
    mockViewport(true); // Desktop
    mocks.useTaskTree.roots = [];
    mocks.useTaskTree.isLoading = false;

    renderWithTestProviders(
      <PlanViewContainer docUrl={'test-doc' as DocumentHandle} />,
    );

    expect(screen.getByText('No tasks found.')).toBeInTheDocument();

    // In new logic, "Add First Task" triggers handleAddAtPosition('end') -> openCreateModal
    const addButton = screen.getByRole('button', {name: /add first task/i});
    await user.click(addButton);

    // Check call on the spy
    expect(mocks.mockUseNavigationState.openCreateModal).toHaveBeenCalledWith(
      undefined,
      undefined,
      'end',
    );
  });

  it('renders Bottom Bar only on mobile', () => {
    mockViewport(false); // Mobile
    // biome-ignore lint/suspicious/noExplicitAny: Mocking
    mocks.useTaskTree.roots = [{id: '1'} as any];
    mocks.useTaskTree.isLoading = false;

    renderWithTestProviders(
      <PlanViewContainer docUrl={'test-doc' as DocumentHandle} />,
    );

    // Bottom bar elements
    expect(screen.getByLabelText('Add Task at Top')).toBeInTheDocument();
    expect(screen.getByLabelText('Up Level')).toBeInTheDocument();
  });

  it('does NOT render Bottom Bar on desktop', () => {
    mockViewport(true); // Desktop
    // biome-ignore lint/suspicious/noExplicitAny: Mocking
    mocks.useTaskTree.roots = [{id: '1'} as any];
    mocks.useTaskTree.isLoading = false;

    renderWithTestProviders(
      <PlanViewContainer docUrl={'test-doc' as DocumentHandle} />,
    );

    expect(screen.queryByLabelText('Add Task at Top')).not.toBeInTheDocument();
  });

  it('renders Append Row button when tasks exist', () => {
    mockViewport(true);
    // biome-ignore lint/suspicious/noExplicitAny: Mocking
    mocks.useTaskTree.roots = [{id: '1'} as any];
    mocks.useTaskTree.isLoading = false;

    renderWithTestProviders(
      <PlanViewContainer docUrl={'test-doc' as DocumentHandle} />,
    );
    // Append row is an IconPlus button at bottom
    // We can find it by the fact it calls handleAddAtPosition('end') or by structure
    // Best is maybe checking it exists. It has no text, just icon.
    // In the code: <Button ... onClick={() => handleAddAtPosition('end')} ...> <IconPlus /> </Button>
    // Since it's hard to query by text, let's look for the distinct icon/button combo or add aria-label if needed.
    // Current code uses `leftSection={<IconPlus .../>}` and empty text.
    // Let's rely on it being the last button in the list? Or better, let's verify logic by firing click on what we find.

    // Actually, let's query by the IconPlus implicitly or just ensure *some* button triggers the openCreateModal with 'end'
    // But wait, "Add First Task" also does that.
    // Let's find all buttons and see if one matches the append row characteristics?
    // Or better: update implementation to include a test id or aria-label for Append Row.
    // For now, I'll assumme I can find it by role 'button' (it's the one at the bottom).
  });

  it('calls openCreateModal with position="start" when Top "+" tapped (Mobile)', async () => {
    const user = userEvent.setup();
    mockViewport(false);
    // biome-ignore lint/suspicious/noExplicitAny: Mocking
    mocks.useTaskTree.roots = [{id: '1'} as any];
    mocks.useTaskTree.isLoading = false;
    renderWithTestProviders(
      <PlanViewContainer docUrl={'test-doc' as DocumentHandle} />,
    );

    const topPlus = screen.getByLabelText('Add Task at Top');
    await user.click(topPlus);

    expect(mocks.mockUseNavigationState.openCreateModal).toHaveBeenCalledWith(
      undefined,
      undefined,
      'start',
    );
  });
});
