import type {TaskID, TunnelNode} from '@mydoo/tasklens';
import {screen} from '@testing-library/react';
import {describe, expect, it, vi} from 'vitest';
import {renderWithTestProviders} from '../../../test/setup';
import {OutlineTree, type OutlineTreeProps} from './OutlineTree';

describe('OutlineTree', () => {
  const mockChild: TunnelNode = {
    id: 'child-1' as TaskID,
    title: 'Child Task',
    status: 'Pending',
    importance: 0.5,
    children: [],
    childTaskIds: [],
    creditIncrement: 1,
    credits: 0,
    creditsTimestamp: 0,
    desiredCredits: 0,
    priorityTimestamp: 0,
    schedule: {type: 'Once', leadTime: 0},
    isAcknowledged: false,
    isSequential: false,
  };

  const mockRoot: TunnelNode = {
    ...mockChild,
    id: 'root-1' as TaskID,
    title: 'Root Task',
    children: [mockChild],
  };

  const defaultProps: OutlineTreeProps = {
    expandedIds: new Set(),
    nodes: [mockRoot],
    onDrillDown: vi.fn(),
    onExpandToggle: vi.fn(),
    onToggleCompletion: vi.fn(),
    onIndent: vi.fn(),
    onOutdent: vi.fn(),
    viewMode: 'tree',
    onOpenEditor: vi.fn(),
    onAddSibling: vi.fn(),
    onAddChild: vi.fn(),
    onDelete: vi.fn(),
    lastCreatedTaskId: undefined,
  };

  const renderComponent = (props: Partial<OutlineTreeProps> = {}) => {
    return renderWithTestProviders(
      <OutlineTree {...defaultProps} {...props} />,
    );
  };

  it('renders root nodes', () => {
    renderComponent();
    expect(screen.getByText('Root Task')).toBeInTheDocument();
    // Child should NOT be visible initially (not expanded)
    expect(screen.queryByText('Child Task')).not.toBeInTheDocument();
  });

  it('renders children when expanded', () => {
    renderComponent({
      expandedIds: new Set(['root-1' as TaskID]),
    });
    expect(screen.getByText('Root Task')).toBeInTheDocument();
    expect(screen.getByText('Child Task')).toBeInTheDocument();
  });

  it('handles empty nodes gracefully', () => {
    renderComponent({nodes: []});
    // MantineProvider renders global styles so container is not empty.
    // We check that no task items are rendered.
    expect(screen.queryByTestId('task-item')).not.toBeInTheDocument();
  });
});
