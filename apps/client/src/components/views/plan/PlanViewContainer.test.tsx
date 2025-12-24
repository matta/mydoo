import {MantineProvider} from '@mantine/core';
import type {DocumentHandle} from '@mydoo/tasklens';
import {fireEvent, render, screen} from '@testing-library/react';
import {describe, expect, it, vi} from 'vitest';
import {PlanViewContainer} from './PlanViewContainer';

// Mock dependencies
const mockCreateTask = vi.fn();
const mockUseTaskIntents = {
  createTask: mockCreateTask,
  toggleTask: vi.fn(),
  deleteTask: vi.fn(),
  indentTask: vi.fn(),
  outdentTask: vi.fn(),
};

const mockUseTaskTree = {
  roots: [],
  isLoading: false,
};

const mockUseNavigationState = {
  currentViewId: undefined,
  expandedIds: new Set(),
  toggleExpanded: vi.fn(),
  pushView: vi.fn(),
  popView: vi.fn(),
  collapseAll: vi.fn(),
  resetView: vi.fn(),
  setViewPath: vi.fn(),
  openEditModal: vi.fn(),
  viewPath: [],
};

const mockUseBreadcrumbs: unknown[] = [];

vi.mock('../../../viewmodel/intents/useTaskIntents', () => ({
  useTaskIntents: () => mockUseTaskIntents,
}));

vi.mock('../../../viewmodel/projections/useTaskTree', () => ({
  useTaskTree: () => mockUseTaskTree,
}));

vi.mock('../../../viewmodel/ui/useNavigationState', () => ({
  useNavigationState: () => mockUseNavigationState,
}));

vi.mock('../../../viewmodel/ui/useBreadcrumbs', () => ({
  useBreadcrumbs: () => mockUseBreadcrumbs,
}));

vi.mock('@mydoo/tasklens', () => ({
  useTunnel: () => ({doc: {tasks: {}}}),
}));

// Mock OutlineTree to simplify rendering
vi.mock('./OutlineTree', () => ({
  OutlineTree: () => <div data-testid="outline-tree" />,
}));

const renderWithProviders = (ui: React.ReactElement) =>
  render(<MantineProvider>{ui}</MantineProvider>);

describe('PlanViewContainer', () => {
  it('renders "Add First Task" button when task list is empty', () => {
    // Setup empty roots
    mockUseTaskTree.roots = [];

    renderWithProviders(
      <PlanViewContainer docUrl={'test-doc' as DocumentHandle} />,
    );

    expect(screen.getByText('No tasks found.')).toBeInTheDocument();
    const addButton = screen.getByRole('button', {name: /add first task/i});
    expect(addButton).toBeInTheDocument();

    // Verify click
    fireEvent.click(addButton);
    expect(mockCreateTask).toHaveBeenCalledWith('New Task');
  });

  // TODO: Add test for "Add First Task" button when drilled down with currentViewId set
});
