'use client';

import { Label } from '@/components/ui/label';
import { Slider } from '@/components/ui/slider';
import { Switch } from '@/components/ui/switch';
import { useState } from 'react';

interface TokenSliderProps {
  minTokens?: number;
  maxTokens?: number;
  initialEnabled?: boolean;
  initialValue?: number;
  isLoading?: boolean;
}

export function TokenSlider({
  minTokens = 0,
  maxTokens = 2048,
  initialEnabled = true,
  initialValue,
  isLoading = false
}: TokenSliderProps) {
  const [isEnabled, setIsEnabled] = useState(initialEnabled);
  const [value, setValue] = useState([initialValue ?? maxTokens]); // Use initialValue if provided, otherwise use maxTokens

  // Determine if interactions should be disabled
  const isDisabled = isLoading || !isEnabled;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <Label className="text-sm font-medium">Max Tokens</Label>
        <div className="flex items-center gap-4">
          <span className={`text-sm text-muted-foreground ${isDisabled ? 'opacity-50' : ''}`}>
            {value[0]}
          </span>
          <Switch
            id="token-slider-toggle"
            checked={isEnabled}
            onCheckedChange={setIsEnabled}
            disabled={isLoading}
          />
        </div>
      </div>
      <Slider
        defaultValue={value}
        max={maxTokens}
        min={minTokens}
        step={1}
        onValueChange={setValue}
        className={`[&_[role=slider]]:h-4 [&_[role=slider]]:w-4 ${isDisabled ? 'opacity-50' : ''}`}
        aria-label="Max tokens"
        disabled={isDisabled}
      />
    </div>
  );
}
