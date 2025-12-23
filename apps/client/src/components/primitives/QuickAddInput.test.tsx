import {fireEvent, screen} from '@testing-library/react';
import {describe, expect, it, vi} from 'vitest';

import {render} from '../../test/setup';
import {QuickAddInput} from './QuickAddInput';

describe('QuickAddInput', () => {
  it('renders with placeholder', () => {
    render(<QuickAddInput onAdd={vi.fn()} />);
    expect(
      screen.getByPlaceholderText('Add a new task...'),
    ).toBeInTheDocument();
  });

  it('calls onAdd with trimmed text on Enter', () => {
    const onAdd = vi.fn();
    render(<QuickAddInput onAdd={onAdd} />);

    const input = screen.getByPlaceholderText('Add a new task...');
    fireEvent.change(input, {target: {value: '  Buy Milk  '}});
    fireEvent.keyDown(input, {key: 'Enter'});

    expect(onAdd).toHaveBeenCalledWith('Buy Milk');
  });

  it('clears input after submit', () => {
    const onAdd = vi.fn();
    render(<QuickAddInput onAdd={onAdd} />);

    const input = screen.getByPlaceholderText('Add a new task...');
    fireEvent.change(input, {target: {value: 'Walk Dog'}});
    fireEvent.keyDown(input, {key: 'Enter'});

    expect(input).toHaveValue('');
  });

  it('does not call onAdd for empty input', () => {
    const onAdd = vi.fn();
    render(<QuickAddInput onAdd={onAdd} />);

    const input = screen.getByPlaceholderText('Add a new task...');
    fireEvent.change(input, {target: {value: '   '}});
    fireEvent.keyDown(input, {key: 'Enter'});

    expect(onAdd).not.toHaveBeenCalled();
  });

  it('calls onAdd when button is clicked', () => {
    const onAdd = vi.fn();
    render(<QuickAddInput onAdd={onAdd} />);

    const input = screen.getByPlaceholderText('Add a new task...');
    fireEvent.change(input, {target: {value: 'Test Task'}});

    const button = screen.getByLabelText('Add task');
    fireEvent.click(button);

    expect(onAdd).toHaveBeenCalledWith('Test Task');
  });

  it('disables button when input is empty', () => {
    render(<QuickAddInput onAdd={vi.fn()} />);

    const button = screen.getByLabelText('Add task');
    expect(button).toBeDisabled();
  });
});
