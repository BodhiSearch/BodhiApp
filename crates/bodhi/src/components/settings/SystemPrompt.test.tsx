import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { SystemPrompt } from './SystemPrompt';

describe('SystemPrompt', () => {
  describe('loading state', () => {
    it('disables all interactive elements when loading', () => {
      render(<SystemPrompt isLoading={true} />);

      const textarea = screen.getByRole('textbox');
      const switchElement = screen.getByRole('switch');

      expect(textarea).toBeDisabled();
      expect(switchElement).toBeDisabled();
    });

    it.skip('prevents interactions while loading', () => {
      render(<SystemPrompt isLoading={true} />);

      const switchElement = screen.getByRole('switch');
      const textarea = screen.getByRole('textbox');

      // Try to toggle switch
      const initialSwitchState = switchElement.getAttribute('aria-checked');
      fireEvent.click(switchElement);
      expect(switchElement.getAttribute('aria-checked')).toBe(initialSwitchState);

      // Try to input text
      fireEvent.change(textarea, { target: { value: 'Test input' } });
      expect(textarea).toHaveValue('');
    });

    it.skip('enables interactions when loading completes', () => {
      const { rerender } = render(<SystemPrompt isLoading={true} />);

      // Initially disabled
      const switchElement = screen.getByRole('switch');
      expect(switchElement).toBeDisabled();

      // Rerender with loading complete
      rerender(<SystemPrompt isLoading={false} initialEnabled={true}/>);

      // Should now be enabled
      expect(switchElement).not.toBeDisabled();
      
      // Enable the textarea
      fireEvent.click(switchElement);
      expect(screen.getByRole('textbox')).not.toBeDisabled();
    });
  });

  describe('initial state', () => {
    it('is enabled by default', () => {
      render(<SystemPrompt />);

      const textarea = screen.getByRole('textbox');
      const switchElement = screen.getByRole('switch');

      expect(switchElement).toHaveAttribute('aria-checked', 'true');
      expect(textarea).not.toBeDisabled();
    });

    it('respects initialEnabled prop when false', () => {
      render(<SystemPrompt initialEnabled={false} />);

      const textarea = screen.getByRole('textbox');
      const switchElement = screen.getByRole('switch');

      expect(switchElement).toHaveAttribute('aria-checked', 'false');
      expect(textarea).toBeDisabled();
    });

    it('respects initialEnabled prop when true', () => {
      render(<SystemPrompt initialEnabled={true} />);

      const textarea = screen.getByRole('textbox');
      const switchElement = screen.getByRole('switch');

      expect(switchElement).toHaveAttribute('aria-checked', 'true');
      expect(textarea).not.toBeDisabled();
    });
  });

  it('renders with switch and textarea', () => {
    render(<SystemPrompt />);

    expect(screen.getByText('System Prompt')).toBeInTheDocument();
    expect(screen.getByRole('switch')).toBeInTheDocument();
    expect(screen.getByRole('textbox')).toBeInTheDocument();
  });

  it('allows text input when enabled', () => {
    render(<SystemPrompt />);

    const textarea = screen.getByRole('textbox');
    fireEvent.change(textarea, { target: { value: 'Test prompt' } });

    expect(textarea).toHaveValue('Test prompt');
  });

  it('maintains text value when toggling switch', () => {
    render(<SystemPrompt />);

    const textarea = screen.getByRole('textbox');
    fireEvent.change(textarea, { target: { value: 'Test prompt' } });

    // Disable
    const switchElement = screen.getByRole('switch');
    fireEvent.click(switchElement);
    expect(textarea).toBeDisabled();
    expect(textarea).toHaveValue('Test prompt');

    // Enable again
    fireEvent.click(switchElement);
    expect(textarea).toBeEnabled();
    expect(textarea).toHaveValue('Test prompt');
  });
});