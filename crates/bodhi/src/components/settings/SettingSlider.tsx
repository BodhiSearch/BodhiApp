'use client';

import { Label } from '@/components/ui/label';
import { Slider } from '@/components/ui/slider';
import { Switch } from '@/components/ui/switch';

interface SettingSliderProps {
  label: string;
  value: number | undefined;
  enabled: boolean;
  onValueChange: (value: number) => void;
  onEnabledChange: (enabled: boolean) => void;
  min?: number;
  max?: number;
  step?: number;
  defaultValue?: number;
  isLoading?: boolean;
}

export function SettingSlider({
  label,
  value,
  enabled,
  onValueChange,
  onEnabledChange,
  min = 0,
  max = 100,
  step = 1,
  defaultValue,
  isLoading = false,
}: SettingSliderProps) {
  // Use provided value or default
  const currentValue = value ?? defaultValue ?? max;

  // Determine if interactions should be disabled
  const isDisabled = isLoading || !enabled;

  const handleValueChange = (values: number[]) => {
    onValueChange(values[0]);
  };

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <Label htmlFor={`setting-${label}`}>{label}</Label>
        <div className="flex items-center gap-4">
          <span
            className={`text-sm text-muted-foreground ${isDisabled ? 'opacity-50' : ''}`}
          >
            {currentValue}
          </span>
          <Switch
            id={`setting-${label}-toggle`}
            checked={enabled}
            onCheckedChange={onEnabledChange}
            disabled={isLoading}
          />
        </div>
      </div>
      <Slider
        id={`setting-${label}`}
        defaultValue={[currentValue]}
        max={max}
        min={min}
        step={step}
        onValueChange={handleValueChange}
        className={`[&_[role=slider]]:h-4 [&_[role=slider]]:w-4 ${isDisabled ? 'opacity-50' : ''}`}
        aria-label={label}
        disabled={isDisabled}
      />
    </div>
  );
}
