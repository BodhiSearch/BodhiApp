import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import { TokenSlider } from '@/components/settings/TokenSlider';
import { useEffect } from 'react';
import * as chatSettings from '@/lib/hooks/use-chat-settings';

// Mock useChatSettings
vi.mock('@/lib/hooks/use-chat-settings', () => ({
  useChatSettings: vi.fn()
}));

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
  beforeEach(() => {
    // Reset mock before each test with default values
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      max_tokens: undefined,
      max_tokens_enabled: true,
      setMaxTokens: vi.fn(),
      setMaxTokensEnabled: vi.fn(),
    } as any);
  });

  it('renders with label and default values', () => {
    render(<TokenSlider />);

    expect(screen.getByText('Max Tokens')).toBeInTheDocument();
    expect(screen.getByText('2048')).toBeInTheDocument();
    expect(screen.getByRole('switch')).toBeInTheDocument();
  });

  it('uses max_tokens from chat settings when available', () => {
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      max_tokens: 1000,
      max_tokens_enabled: true,
      setMaxTokens: vi.fn(),
      setMaxTokensEnabled: vi.fn(),
    } as any);

    render(<TokenSlider />);
    expect(screen.getByText('1000')).toBeInTheDocument();
    expect(screen.getByRole('slider')).toHaveValue('1000');
  });

  it('uses maxTokens prop as default when max_tokens is undefined', () => {
    render(<TokenSlider maxTokens={4096} />);
    expect(screen.getByText('4096')).toBeInTheDocument();
    expect(screen.getByRole('slider')).toHaveValue('4096');
  });

  it.skip('constrains value within min and max bounds', () => {
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      max_tokens: 4000,
      max_tokens_enabled: true,
      setMaxTokens: vi.fn(),
      setMaxTokensEnabled: vi.fn(),
    } as any);

    render(<TokenSlider minTokens={500} maxTokens={3000} />);
    expect(screen.getByText('3000')).toBeInTheDocument();
    expect(screen.getByRole('slider')).toHaveValue('3000');
  });

  it('reflects enabled state from chat settings', () => {
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      max_tokens: 2048,
      max_tokens_enabled: false,
      setMaxTokens: vi.fn(),
      setMaxTokensEnabled: vi.fn(),
    } as any);

    render(<TokenSlider />);

    const slider = screen.getByRole('slider');
    expect(slider).toBeDisabled();
    expect(screen.getByRole('switch')).toHaveAttribute('aria-checked', 'false');
  });

  it('updates max_tokens in chat settings when slider changes', () => {
    const mockSetMaxTokens = vi.fn();
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      max_tokens: 2048,
      max_tokens_enabled: true,
      setMaxTokens: mockSetMaxTokens,
      setMaxTokensEnabled: vi.fn(),
    } as any);

    render(<TokenSlider />);

    const slider = screen.getByRole('slider');
    fireEvent.change(slider, { target: { value: '1000' } });

    expect(mockSetMaxTokens).toHaveBeenCalledWith(1000);
  });

  it('updates enabled state in chat settings', () => {
    const mockSetEnabled = vi.fn();
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      max_tokens: 2048,
      max_tokens_enabled: true,
      setMaxTokens: vi.fn(),
      setMaxTokensEnabled: mockSetEnabled,
    } as any);

    render(<TokenSlider />);

    const switchElement = screen.getByRole('switch');
    fireEvent.click(switchElement);

    expect(mockSetEnabled).toHaveBeenCalledWith(false);
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
      const mockSetMaxTokens = vi.fn();
      const mockSetEnabled = vi.fn();

      vi.mocked(chatSettings.useChatSettings).mockReturnValue({
        max_tokens: 1000,
        max_tokens_enabled: true,
        setMaxTokens: mockSetMaxTokens,
        setMaxTokensEnabled: mockSetEnabled,
      } as any);

      render(<TokenSlider isLoading={true} />);

      const slider = screen.getByRole('slider');
      const switchElement = screen.getByRole('switch');

      // Try to change slider value
      fireEvent.change(slider, { target: { value: '1500' } });
      expect(mockSetMaxTokens).not.toHaveBeenCalled();

      // Try to toggle switch
      fireEvent.click(switchElement);
      expect(mockSetEnabled).not.toHaveBeenCalled();
    });

    it.skip('applies opacity styling when disabled', () => {
      render(<TokenSlider isLoading={true} />);

      const slider = screen.getByRole('group');
      const valueDisplay = screen.getByText('2048');

      expect(slider.className).toContain('opacity-50');
      expect(valueDisplay.parentElement?.className).toContain('opacity-50');
    });
  });
});