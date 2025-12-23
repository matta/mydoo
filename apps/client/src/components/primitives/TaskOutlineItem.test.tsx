import {MantineProvider} from '@mantine/core';
import type {TaskID, TunnelNode} from '@mydoo/tasklens';
import {render, screen} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {describe, expect, it, vi} from 'vitest';
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
  };

  const defaultProps: TaskOutlineItemProps = {
    depth: 0,
    isExpanded: false,
    node: mockNode,
    onDrillDown: vi.fn(),
    onExpandToggle: vi.fn(),
    onToggleCompletion: vi.fn(),
  };

  const renderComponent = (props: Partial<TaskOutlineItemProps> = {}) => {
    return render(
      <MantineProvider>
        <TaskOutlineItem {...defaultProps} {...props} />
      </MantineProvider>,
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
    renderComponent({onDrillDown});

    const drillBtn = screen.getByRole('button', {name: 'Focus view'});
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
});
