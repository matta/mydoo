import {screen} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {describe, expect, it, vi} from 'vitest';

import {renderWithTestProviders} from '../../test/setup';
import {QuickAddInput} from './QuickAddInput';

describe('QuickAddInput', () => {
  it('renders with placeholder', () => {
    renderWithTestProviders(<QuickAddInput onAdd={vi.fn()} />);
    expect(
      screen.getByPlaceholderText('Add a new task...'),
    ).toBeInTheDocument();
  });

  it('calls onAdd with trimmed text on Enter', async () => {
    const user = userEvent.setup();
    const onAdd = vi.fn();
    renderWithTestProviders(<QuickAddInput onAdd={onAdd} />);

    const input = screen.getByPlaceholderText('Add a new task...');
    await user.type(input, '  Buy Milk  {Enter}');

    expect(onAdd).toHaveBeenCalledWith('Buy Milk');
  });

  it('clears input after submit', async () => {
    const user = userEvent.setup();
    const onAdd = vi.fn();
    renderWithTestProviders(<QuickAddInput onAdd={onAdd} />);

    const input = screen.getByPlaceholderText('Add a new task...');
    await user.type(input, 'Walk Dog{Enter}');

    expect(input).toHaveValue('');
  });

  it('does not call onAdd for empty input', async () => {
    const user = userEvent.setup();
    const onAdd = vi.fn();
    renderWithTestProviders(<QuickAddInput onAdd={onAdd} />);

    const input = screen.getByPlaceholderText('Add a new task...');
    await user.type(input, '   {Enter}');

    expect(onAdd).not.toHaveBeenCalled();
  });

  it('calls onAdd when button is clicked', async () => {
    const user = userEvent.setup();
    const onAdd = vi.fn();
    renderWithTestProviders(<QuickAddInput onAdd={onAdd} />);

    const input = screen.getByPlaceholderText('Add a new task...');
    await user.type(input, 'Test Task');

    const button = screen.getByLabelText('Add task');
    await user.click(button);

    expect(onAdd).toHaveBeenCalledWith('Test Task');
  });

  it('disables button when input is empty', () => {
    renderWithTestProviders(<QuickAddInput onAdd={vi.fn()} />);

    const button = screen.getByLabelText('Add task');
    expect(button).toBeDisabled();
  });
});
