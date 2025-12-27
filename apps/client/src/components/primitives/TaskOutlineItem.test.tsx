import type {TaskID, TunnelNode} from '@mydoo/tasklens';
import {screen} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {describe, expect, it, vi} from 'vitest';
import {renderWithTestProviders} from '../../test/setup';
import {TaskOutlineItem, type TaskOutlineItemProps} from './TaskOutlineItem';

describe('TaskOutlineItem', () => {
  const mockNode: TunnelNode = {
    id: 'task-1' as TaskID,
    title: 'Test Task',
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
    notes: '',
  };

  const defaultProps: TaskOutlineItemProps = {
    depth: 0,
    isExpanded: false,
    node: mockNode,
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
  };

  const renderComponent = (props: Partial<TaskOutlineItemProps> = {}) => {
    return renderWithTestProviders(
      <TaskOutlineItem {...defaultProps} {...props} />,
    );
  };

  it('renders task title', () => {
    renderComponent();
    expect(screen.getByText('Test Task')).toBeInTheDocument();
  });

  it('renders completion checkbox and calls toggle handler', async () => {
    const onToggleCompletion = vi.fn();
    renderComponent({onToggleCompletion});

    const checkbox = screen.getByRole('checkbox', {
      name: /Complete Test Task/i,
    });
    expect(checkbox).toBeInTheDocument();

    await userEvent.click(checkbox);
    expect(onToggleCompletion).toHaveBeenCalled();
  });

  it('renders expansion chevron and calls handler when node has children', async () => {
    const onExpandToggle = vi.fn();
    const childNode = {...mockNode, id: 'child-1' as TaskID};
    const parentNode = {...mockNode, children: [childNode]};

    renderComponent({
      node: parentNode,
      onExpandToggle,
      isExpanded: false,
    });

    const expandBtn = screen.getByRole('button', {name: 'Toggle expansion'});
    expect(expandBtn).toBeVisible();

    await userEvent.click(expandBtn);
    expect(onExpandToggle).toHaveBeenCalled();
  });

  it('renders drill-down button and calls handler', async () => {
    const onDrillDown = vi.fn();
    const childNode = {...mockNode, id: 'child-1' as TaskID};
    const parentNode = {...mockNode, children: [childNode]};
    renderComponent({onDrillDown, viewMode: 'drill', node: parentNode});

    const drillBtn = screen.getByRole('button', {name: 'Drill down'});
    expect(drillBtn).toBeInTheDocument();

    await userEvent.click(drillBtn);
    expect(onDrillDown).toHaveBeenCalled();
  });

  it('applies depth padding correctly', () => {
    renderComponent({depth: 2});
    const groupDiv = screen.getByTestId('task-item');
    expect(groupDiv.style.paddingLeft).toContain(
      'calc(2 * var(--mantine-spacing-md))',
    );
  });

  it('calls onIndent when Tab is pressed', async () => {
    const onIndent = vi.fn();
    renderComponent({onIndent});

    const row = screen.getByTestId('task-item');
    await userEvent.tab(); // Focus the element?
    // userEvent.tab() just creates a tab event, might not focus row if not set up.
    // We added tabIndex={0} to Group.
    row.focus();
    await userEvent.keyboard('{Tab}');

    expect(onIndent).toHaveBeenCalled();
  });

  it('calls onOutdent when Shift+Tab is pressed', async () => {
    const onOutdent = vi.fn();
    renderComponent({onOutdent});

    const row = screen.getByTestId('task-item');
    row.focus();
    await userEvent.keyboard('{Shift>}{Tab}{/Shift}');

    expect(onOutdent).toHaveBeenCalled();
  });

  it('calls onAddSibling when menu action clicked', async () => {
    const onAddSibling = vi.fn();
    renderComponent({onAddSibling, viewMode: 'tree'});

    await userEvent.click(screen.getByTestId('task-menu-trigger'));
    // Menu dropdown renders in a portal; query within document body
    const menuItem = await screen.findByRole('menuitem', {
      name: /add sibling/i,
    });
    await userEvent.click(menuItem);

    expect(onAddSibling).toHaveBeenCalled();
  });

  it('calls onAddChild when menu action clicked', async () => {
    const onAddChild = vi.fn();
    renderComponent({onAddChild, viewMode: 'tree'});

    await userEvent.click(screen.getByTestId('task-menu-trigger'));
    const menuItem = await screen.findByRole('menuitem', {name: /add child/i});
    await userEvent.click(menuItem);

    expect(onAddChild).toHaveBeenCalled();
  });

  it('calls onDelete when menu action clicked', async () => {
    const onDelete = vi.fn();
    renderComponent({onDelete, viewMode: 'tree'});

    await userEvent.click(screen.getByTestId('task-menu-trigger'));
    const menuItem = await screen.findByRole('menuitem', {name: /delete/i});
    await userEvent.click(menuItem);

    expect(onDelete).toHaveBeenCalled();
  });

  it('renders menu trigger in tree mode', () => {
    // Menu trigger should be present in DOM (opacity controlled by CSS)
    renderComponent({viewMode: 'tree'});
    const trigger = screen.getByTestId('task-menu-trigger');
    expect(trigger).toBeInTheDocument();
  });

  it('renders menu trigger in drill mode (for mobile context menu)', () => {
    renderComponent({viewMode: 'drill'});
    const trigger = screen.getByTestId('task-menu-trigger');
    expect(trigger).toBeInTheDocument();
  });
});
