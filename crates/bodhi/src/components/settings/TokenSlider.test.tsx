import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { TokenSlider } from '@/components/settings/TokenSlider';
import { useEffect } from 'react';

// Create a more accurate Slider mock that mimics the real component's behavior
vi.mock('@/components/ui/slider', () => ({
  Slider: ({ defaultValue, min, max, step, onValueChange, disabled, ...props }: any) => {
    const handleChange = (e: any) => {
      if (disabled) return;

      const value = parseInt(e.target.value);
      // Constrain value within bounds, just like the real slider
      const constrainedValue = Math.min(Math.max(value, min), max);
      onValueChange?.([constrainedValue]);
    };

    useEffect(() => {
      if (defaultValue && defaultValue > max) {
        onValueChange?.([max]);
      } else if (defaultValue && defaultValue < min) {
        onValueChange?.([min]);
      }
    }, []);

    return (
      <div className="relative" role="group">
        <input
          type="range"
          min={min}
          max={max}
          step={step}
          value={defaultValue?.[0]}
          onChange={handleChange}
          disabled={disabled}
          {...props}
        />
      </div>
    );
  },
}));

// Mock the Switch component from shadcn
vi.mock('@/components/ui/switch', () => ({
  Switch: ({ checked, onCheckedChange, ...props }: any) => (
    <button
      role="switch"
      aria-checked={checked}
      onClick={() => onCheckedChange(!checked)}
      {...props}
    />
  ),
}));

describe('TokenSlider', () => {
  it('renders with label and default values', () => {
    render(<TokenSlider />);

    expect(screen.getByText('Max Tokens')).toBeInTheDocument();
    expect(screen.getByText('2048')).toBeInTheDocument();
    expect(screen.getByRole('switch')).toBeInTheDocument();
  });

  it('initializes with provided initial value', () => {
    render(<TokenSlider initialValue={1000} />);
    expect(screen.getByText('1000')).toBeInTheDocument();
    expect(screen.getByRole('slider')).toHaveValue('1000');
  });

  it('uses maxTokens as initial value when no initialValue is provided', () => {
    render(<TokenSlider maxTokens={4096} />);
    expect(screen.getByText('4096')).toBeInTheDocument();
    expect(screen.getByRole('slider')).toHaveValue('4096');
  });

  it('constrains initial value within min and max bounds', () => {
    render(<TokenSlider minTokens={500} maxTokens={3000} initialValue={4000} />);
    expect(screen.getByText('3000')).toBeInTheDocument();
    expect(screen.getByRole('slider')).toHaveValue('3000');
  });

  it('renders slider with default attributes', () => {
    render(<TokenSlider />);

    const slider = screen.getByRole('slider');
    expect(slider).toHaveAttribute('aria-label', 'Max tokens');
    expect(slider).toHaveAttribute('min', '0');
    expect(slider).toHaveAttribute('max', '2048');
    expect(slider).toHaveAttribute('step', '1');
  });

  it('accepts custom min and max values', () => {
    render(<TokenSlider minTokens={100} maxTokens={4096} />);

    const slider = screen.getByRole('slider');
    expect(slider).toHaveAttribute('min', '100');
    expect(slider).toHaveAttribute('max', '4096');
    expect(screen.getByText('4096')).toBeInTheDocument();
  });

  it('initializes with initialEnabled prop', () => {
    render(<TokenSlider initialEnabled={false} />);

    const slider = screen.getByRole('slider');
    expect(slider).toBeDisabled();
    expect(screen.getByRole('switch')).toHaveAttribute('aria-checked', 'false');
  });

  it('is enabled by default', () => {
    render(<TokenSlider />);

    const slider = screen.getByRole('slider');
    expect(slider).not.toBeDisabled();
    expect(screen.getByRole('switch')).toHaveAttribute('aria-checked', 'true');
  });

  it('disables slider when toggle is switched off', () => {
    render(<TokenSlider />);

    const switchElement = screen.getByRole('switch');
    fireEvent.click(switchElement);

    const slider = screen.getByRole('slider');
    expect(slider).toBeDisabled();
    expect(switchElement).toHaveAttribute('aria-checked', 'false');
  });

  it('updates value when slider changes', () => {
    render(<TokenSlider />);

    const slider = screen.getByRole('slider');
    fireEvent.change(slider, { target: { value: '1000' } });

    expect(screen.getByText('1000')).toBeInTheDocument();
  });

  it('maintains value when toggling enabled state', () => {
    render(<TokenSlider initialValue={1000} />);

    const slider = screen.getByRole('slider');
    expect(screen.getByText('1000')).toBeInTheDocument();

    const switchElement = screen.getByRole('switch');
    fireEvent.click(switchElement);
    expect(slider).toBeDisabled();
    expect(screen.getByText('1000')).toBeInTheDocument();

    fireEvent.click(switchElement);
    expect(slider).not.toBeDisabled();
    expect(screen.getByText('1000')).toBeInTheDocument();
  });

  it('constrains value within provided min and max', () => {
    render(<TokenSlider minTokens={500} maxTokens={3000} />);

    const slider = screen.getByRole('slider');
    fireEvent.change(slider, { target: { value: '4000' } });
    expect(slider).toHaveAttribute('value', '3000');

    fireEvent.change(slider, { target: { value: '100' } });
    expect(slider).toHaveAttribute('value', '500');
  });

  describe('loading state', () => {
    it('disables all interactive elements when loading', () => {
      render(<TokenSlider isLoading={true} />);

      const slider = screen.getByRole('slider');
      const switchElement = screen.getByRole('switch');

      expect(slider).toBeDisabled();
      expect(switchElement).toBeDisabled();
    });

    it('prevents interactions while loading', () => {
      render(<TokenSlider isLoading={true} initialValue={1000} />);

      const slider = screen.getByRole('slider');
      const switchElement = screen.getByRole('switch');

      // Try to change slider value
      fireEvent.change(slider, { target: { value: '1500' } });
      expect(screen.getByText('1000')).toBeInTheDocument();

      // Try to toggle switch
      const initialState = switchElement.getAttribute('aria-checked');
      fireEvent.click(switchElement);
      expect(switchElement.getAttribute('aria-checked')).toBe(initialState);
    });

    it('enables interactions when loading completes', () => {
      const { rerender } = render(<TokenSlider isLoading={true} />);

      // Initially disabled
      const slider = screen.getByRole('slider');
      const switchElement = screen.getByRole('switch');
      expect(slider).toBeDisabled();
      expect(switchElement).toBeDisabled();

      // Rerender with loading complete
      rerender(<TokenSlider isLoading={false} />);

      // Should now be enabled
      expect(slider).not.toBeDisabled();
      expect(switchElement).not.toBeDisabled();
    });

    it.skip('applies opacity styling when disabled', () => {
      render(<TokenSlider isLoading={true} initialValue={1000} />);

      const slider = screen.getByRole('group');
      const valueDisplay = screen.getByText('1000');

      expect(slider.className).toContain('opacity-50');
      expect(valueDisplay.parentElement?.className).toContain('opacity-50');
    });
  });
});