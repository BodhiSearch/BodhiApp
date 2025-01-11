import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import { SettingSlider } from '@/components/settings/SettingSlider';
import { useEffect } from 'react';

// Mock the Slider component
vi.mock('@/components/ui/slider', () => ({
  Slider: ({ defaultValue, min, max, step, onValueChange, disabled, ...props }: any) => {
    const handleChange = (e: any) => {
      if (disabled) return;

      const value = parseInt(e.target.value);
      const constrainedValue = Math.min(Math.max(value, min), max);
      onValueChange?.([constrainedValue]);
    };

    useEffect(() => {
      if (defaultValue && defaultValue[0] > max) {
        onValueChange?.([max]);
      } else if (defaultValue && defaultValue[0] < min) {
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

// Mock the Switch component
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

describe('SettingSlider', () => {
  const defaultProps = {
    label: 'Test Setting',
    value: 50,
    enabled: true,
    onValueChange: vi.fn(),
    onEnabledChange: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders with label and value', () => {
    render(<SettingSlider {...defaultProps} />);

    expect(screen.getByText('Test Setting')).toBeInTheDocument();
    expect(screen.getByText('50')).toBeInTheDocument();
    expect(screen.getByRole('switch')).toBeInTheDocument();
  });

  it('uses default value when value is undefined', () => {
    render(<SettingSlider {...defaultProps} value={undefined} defaultValue={75} />);
    expect(screen.getByText('75')).toBeInTheDocument();
    expect(screen.getByRole('slider')).toHaveValue('75');
  });

  it('uses max value as default when no value or defaultValue provided', () => {
    render(<SettingSlider {...defaultProps} value={undefined} max={200} />);
    expect(screen.getByText('200')).toBeInTheDocument();
    expect(screen.getByRole('slider')).toHaveValue('200');
  });

  it('updates value when slider changes', () => {
    const onValueChange = vi.fn();
    render(<SettingSlider {...defaultProps} onValueChange={onValueChange} />);

    const slider = screen.getByRole('slider');
    fireEvent.change(slider, { target: { value: '75' } });

    expect(onValueChange).toHaveBeenCalledWith(75);
  });

  it('updates enabled state when switch is toggled', () => {
    const onEnabledChange = vi.fn();
    render(<SettingSlider {...defaultProps} onEnabledChange={onEnabledChange} />);

    const switchElement = screen.getByRole('switch');
    fireEvent.click(switchElement);

    expect(onEnabledChange).toHaveBeenCalledWith(false);
  });

  describe('loading and disabled states', () => {
    it('disables all interactive elements when loading', () => {
      render(<SettingSlider {...defaultProps} isLoading={true} />);

      const slider = screen.getByRole('slider');
      const switchElement = screen.getByRole('switch');

      expect(slider).toBeDisabled();
      expect(switchElement).toBeDisabled();
    });

    it('disables slider when enabled is false', () => {
      render(<SettingSlider {...defaultProps} enabled={false} />);

      const slider = screen.getByRole('slider');
      expect(slider).toBeDisabled();
    });

    it('prevents interactions while loading', () => {
      const onValueChange = vi.fn();
      const onEnabledChange = vi.fn();

      render(
        <SettingSlider
          {...defaultProps}
          isLoading={true}
          onValueChange={onValueChange}
          onEnabledChange={onEnabledChange}
        />
      );

      const slider = screen.getByRole('slider');
      const switchElement = screen.getByRole('switch');

      fireEvent.change(slider, { target: { value: '75' } });
      expect(onValueChange).not.toHaveBeenCalled();

      fireEvent.click(switchElement);
      expect(onEnabledChange).not.toHaveBeenCalled();
    });
  });
}); 