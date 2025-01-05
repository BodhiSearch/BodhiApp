import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { StopWords } from '@/components/settings/StopWords';

// Example of mocking Input component with proper disabled state handling
vi.mock('@/components/ui/input', () => ({
  Input: ({ disabled, onChange, onKeyDown, ...props }: any) => (
    <input
      {...props}
      disabled={disabled}
      data-disabled={disabled}
      onChange={(e) => !disabled && onChange?.(e)}
      onKeyDown={(e) => !disabled && onKeyDown?.(e)}
    />
  ),
}));

// Mock the Switch component from shadcn
vi.mock('@/components/ui/switch', () => ({
  Switch: ({ checked, onCheckedChange, disabled, ...props }: any) => (
    <button
      role="switch"
      aria-checked={checked}
      onClick={() => !disabled && onCheckedChange(!checked)}
      disabled={disabled}
      {...props}
    />
  ),
}));

describe('StopWords', () => {
  // New test for loading state
  describe('loading state', () => {
    it('disables all interactive elements when loading', () => {
      render(<StopWords isLoading={true} initialStopWords={['test']} />);

      // Check input is disabled
      const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
      expect(input).toBeDisabled();

      // Check switch is disabled
      const switchElement = screen.getByRole('switch');
      expect(switchElement).toBeDisabled();

      // Check remove buttons are disabled
      const removeButton = screen.getByLabelText('Remove test');
      expect(removeButton).toBeDisabled();
    });

    it('prevents interactions while loading', () => {
      render(<StopWords isLoading={true} />);

      const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
      const switchElement = screen.getByRole('switch');

      // Try to add a word
      fireEvent.change(input, { target: { value: 'test' } });
      fireEvent.keyDown(input, { key: 'Enter' });
      expect(screen.queryByText('test')).not.toBeInTheDocument();

      // Try to toggle switch
      const initialState = switchElement.getAttribute('aria-checked');
      fireEvent.click(switchElement);
      expect(switchElement.getAttribute('aria-checked')).toBe(initialState);
    });

    it('enables interactions when loading completes', () => {
      const { rerender } = render(<StopWords isLoading={true} />);

      // Initially disabled
      const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
      expect(input).toBeDisabled();

      // Rerender with loading complete
      rerender(<StopWords isLoading={false} />);

      // Should now be enabled
      expect(input).not.toBeDisabled();
      expect(screen.getByRole('switch')).not.toBeDisabled();
    });
  });

  it('renders input with label and toggle switch', () => {
    render(<StopWords />);

    expect(screen.getByText('Stop Words')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Type and press Enter to add stop words...')).toBeInTheDocument();
    expect(screen.getByRole('switch')).toBeInTheDocument();
  });

  it('renders with initial stop words', () => {
    const initialWords = ['hello', 'world'];
    render(<StopWords initialStopWords={initialWords} />);

    initialWords.forEach(word => {
      expect(screen.getByText(word)).toBeInTheDocument();
      expect(screen.getByLabelText(`Remove ${word}`)).toBeInTheDocument();
    });
  });

  it('initializes with initialEnabled prop', () => {
    render(<StopWords initialEnabled={false} />);

    const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
    expect(input).toBeDisabled();
    expect(screen.getByRole('switch')).toHaveAttribute('aria-checked', 'false');
  });

  it('allows removing initial stop words when enabled', () => {
    const initialWords = ['hello', 'world'];
    render(<StopWords initialStopWords={initialWords} />);

    const removeButton = screen.getByLabelText('Remove hello');
    fireEvent.click(removeButton);

    expect(screen.queryByText('hello')).not.toBeInTheDocument();
    expect(screen.getByText('world')).toBeInTheDocument();
  });

  it('is enabled by default', () => {
    render(<StopWords />);

    const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
    expect(input).not.toBeDisabled();
    expect(screen.getByRole('switch')).toHaveAttribute('aria-checked', 'true');
  });

  it('disables input when toggle is switched off', () => {
    render(<StopWords />);

    const switchElement = screen.getByRole('switch');
    fireEvent.click(switchElement);

    const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
    expect(input).toBeDisabled();
    expect(switchElement).toHaveAttribute('aria-checked', 'false');
  });

  it('maintains stop words when toggling enabled state', () => {
    const initialWords = ['initial'];
    render(<StopWords initialStopWords={initialWords} />);

    // Add another word
    const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
    fireEvent.change(input, { target: { value: 'test' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    // Disable
    const switchElement = screen.getByRole('switch');
    fireEvent.click(switchElement);

    // Words should still be visible
    expect(screen.getByText('initial')).toBeInTheDocument();
    expect(screen.getByText('test')).toBeInTheDocument();
    expect(screen.getByLabelText('Remove initial')).toBeDisabled();
    expect(screen.getByLabelText('Remove test')).toBeDisabled();

    // Enable again
    fireEvent.click(switchElement);
    expect(screen.getByText('initial')).toBeInTheDocument();
    expect(screen.getByText('test')).toBeInTheDocument();
    expect(screen.getByLabelText('Remove initial')).not.toBeDisabled();
    expect(screen.getByLabelText('Remove test')).not.toBeDisabled();
  });

  it('prevents adding stop words when disabled', () => {
    render(<StopWords />);

    // Disable the component
    const switchElement = screen.getByRole('switch');
    fireEvent.click(switchElement);

    // Try to add a word
    const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
    fireEvent.change(input, { target: { value: 'test' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    // Word should not be added
    expect(screen.queryByText('test')).not.toBeInTheDocument();
  });

  it('adds new stop word on Enter when enabled', () => {
    render(<StopWords />);

    const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
    fireEvent.change(input, { target: { value: 'test' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(screen.getByText('test')).toBeInTheDocument();
    expect(input).toHaveValue('');
  });

  it('prevents duplicate stop words', () => {
    render(<StopWords initialStopWords={['test']} />);

    const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
    fireEvent.change(input, { target: { value: 'test' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(screen.getAllByText('test')).toHaveLength(1);
  });

  it('removes stop word when clicking X button while enabled', () => {
    render(<StopWords />);

    // Add a stop word
    const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
    fireEvent.change(input, { target: { value: 'test' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    // Click remove button
    const removeButton = screen.getByLabelText('Remove test');
    fireEvent.click(removeButton);

    expect(screen.queryByText('test')).not.toBeInTheDocument();
  });

  it('ignores empty input', () => {
    render(<StopWords />);

    const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
    fireEvent.change(input, { target: { value: '   ' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(screen.queryByRole('button', { name: /Remove/ })).not.toBeInTheDocument();
  });
});