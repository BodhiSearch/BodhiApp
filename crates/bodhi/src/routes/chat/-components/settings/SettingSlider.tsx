import { Label } from '@/components/ui/label';
import { Slider } from '@/components/ui/slider';
import { Switch } from '@/components/ui/switch';
import { cn } from '@/lib/utils';

import { HelpTooltip } from './HelpTooltip';

interface SettingSliderProps {
  label: string;
  tooltip?: string;
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
  tooltip,
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
  const currentValue = value ?? defaultValue ?? max;
  const isDisabled = isLoading || !enabled;
  const id = `setting-${label.toLowerCase().replace(/\s+/g, '-')}`;

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Label htmlFor={id}>{label}</Label>
          {tooltip && <HelpTooltip text={tooltip} sideOffset={8} />}
        </div>
        <div className="flex items-center gap-2">
          <span className={cn('text-sm tabular-nums text-muted-foreground', isDisabled && 'opacity-50')}>
            {currentValue}
          </span>
          <Switch
            id={`${id}-toggle`}
            data-testid={`${id}-toggle`}
            checked={enabled}
            onCheckedChange={(checked) => {
              if (checked && value == null) {
                onValueChange(currentValue);
              }
              onEnabledChange(checked);
            }}
            disabled={isLoading}
            size="sm"
          />
        </div>
      </div>
      {enabled && (
        <Slider
          id={id}
          defaultValue={[currentValue]}
          max={max}
          min={min}
          step={step}
          onValueChange={(values) => onValueChange(values[0])}
          disabled={isLoading}
          className="[&_[role=slider]]:h-4 [&_[role=slider]]:w-4"
          aria-label={label}
        />
      )}
    </div>
  );
}
