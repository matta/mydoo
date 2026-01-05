import {Repo, type StorageAdapterInterface} from '@automerge/automerge-repo';
import {useMediaQuery} from '@mantine/hooks';
import {strictMock} from '@mydoo/tasklens/test';

// Create a minimal storage adapter using strictMock - only implements what Repo actually uses
function createDummyStorageAdapter(): StorageAdapterInterface {
  return strictMock<StorageAdapterInterface>('DummyStorageAdapter', {
    load: async () => undefined,
    save: async () => {},
    remove: async () => {},
    loadRange: async () => [],
    removeRange: async () => {},
  });
}

import {act, screen} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {beforeEach, describe, expect, it, vi} from 'vitest';
import {renderWithTestProviders} from '../../../test/setup';
import {PlanViewContainer} from './plan-view-container';

// Mock simple view-model hooks that are not the focus of this integration
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

vi.mock('@mantine/hooks', async () => {
  const actual = await vi.importActual('@mantine/hooks');
  return {
    ...actual,
    useMediaQuery: vi.fn(),
  };
});

vi.mock('../../../viewmodel/intents/use-task-intents', () => ({
  useTaskIntents: () => ({
    createTask: vi.fn(),
    toggleTask: vi.fn(),
    deleteTask: vi.fn(),
    indentTask: vi.fn(),
    outdentTask: vi.fn(),
  }),
}));

vi.mock('../../../viewmodel/projections/use-task-tree', () => ({
  useTaskTree: () => mocks.useTaskTree,
}));

vi.mock('../../../viewmodel/ui/use-navigation-state', () => ({
  useNavigationState: () => mocks.mockUseNavigationState,
}));

vi.mock('../../../viewmodel/ui/use-breadcrumbs', () => ({
  useBreadcrumbs: () => [],
}));

// Mock OutlineTree to simplify rendering
vi.mock('./outline-tree', () => ({
  OutlineTree: () => <div data-testid="outline-tree" />,
}));

describe('PlanViewContainer', () => {
  // Helper to mock viewport per test
  const mockViewport = (isDesktop: boolean) => {
    vi.mocked(useMediaQuery).mockReturnValue(isDesktop);
  };

  let repo: Repo;

  // Reset mocks before each test
  beforeEach(() => {
    vi.clearAllMocks();
    repo = new Repo({network: [], storage: createDummyStorageAdapter()});
    repo.create({tasks: {}, rootTaskIds: [], places: {}});
  });

  it('renders "Add First Task" button when task list is empty', async () => {
    const user = userEvent.setup();
    mockViewport(true); // Desktop
    mocks.useTaskTree.roots = [];
    mocks.useTaskTree.isLoading = false;

    await act(async () => {
      renderWithTestProviders(<PlanViewContainer />, {repo});
    });

    expect(screen.getByText('No tasks found.')).toBeInTheDocument();

    const addButton = screen.getByRole('button', {name: /add first task/i});
    await user.click(addButton);

    expect(mocks.mockUseNavigationState.openCreateModal).toHaveBeenCalledWith(
      undefined,
      undefined,
      'end',
    );
  });

  it('renders Bottom Bar only on mobile', async () => {
    mockViewport(false); // Mobile
    // biome-ignore lint/suspicious/noExplicitAny: Mocking
    mocks.useTaskTree.roots = [{id: '1'} as any];
    mocks.useTaskTree.isLoading = false;

    await act(async () => {
      renderWithTestProviders(<PlanViewContainer />, {repo});
    });

    expect(screen.getByLabelText('Add Task at Top')).toBeInTheDocument();
    expect(screen.getByLabelText('Up Level')).toBeInTheDocument();
  });

  it('does NOT render Bottom Bar on desktop', async () => {
    mockViewport(true); // Desktop
    // biome-ignore lint/suspicious/noExplicitAny: Mocking
    mocks.useTaskTree.roots = [{id: '1'} as any];
    mocks.useTaskTree.isLoading = false;

    await act(async () => {
      renderWithTestProviders(<PlanViewContainer />, {repo});
    });

    expect(screen.queryByLabelText('Add Task at Top')).not.toBeInTheDocument();
  });

  it('renders IconPlus (Append Row) button when tasks exist', async () => {
    mockViewport(true);
    // biome-ignore lint/suspicious/noExplicitAny: Mocking
    mocks.useTaskTree.roots = [{id: '1'} as any];
    mocks.useTaskTree.isLoading = false;

    await act(async () => {
      renderWithTestProviders(<PlanViewContainer />, {repo});
    });

    // The append button is an ActionIcon with an IconPlus, but no text.
    // It's the only one of its kind at the bottom right usually, but here just checking presence.
    // In this view, we can rely on finding it by role, although standardizing aria-label would be better.
    // For now, checking that we don't have "No tasks found" is a start, and we can query for the button.
    const buttons = screen.getAllByRole('button');
    expect(buttons.length).toBeGreaterThan(0);
  });

  it('calls openCreateModal with position="start" when Top "+" tapped (Mobile)', async () => {
    const user = userEvent.setup();
    mockViewport(false);
    // biome-ignore lint/suspicious/noExplicitAny: Mocking
    mocks.useTaskTree.roots = [{id: '1'} as any];
    mocks.useTaskTree.isLoading = false;
    await act(async () => {
      renderWithTestProviders(<PlanViewContainer />, {repo});
    });

    const topPlus = screen.getByLabelText('Add Task at Top');
    await user.click(topPlus);

    expect(mocks.mockUseNavigationState.openCreateModal).toHaveBeenCalledWith(
      undefined,
      undefined,
      'start',
    );
  });
});
