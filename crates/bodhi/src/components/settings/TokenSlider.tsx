'use client';

import { Label } from '@/components/ui/label';
import { Slider } from '@/components/ui/slider';
import { Switch } from '@/components/ui/switch';
import { useChatSettings } from '@/lib/hooks/use-chat-settings';

interface TokenSliderProps {
  minTokens?: number;
  maxTokens?: number;
  isLoading?: boolean;
}

export function TokenSlider({
  minTokens = 0,
  maxTokens = 2048,
  isLoading = false
}: TokenSliderProps) {
  const { 
    max_tokens,
    max_tokens_enabled,
    setMaxTokens,
    setMaxTokensEnabled
  } = useChatSettings();

  // Use max_tokens from settings or maxTokens prop as default
  const currentValue = max_tokens ?? maxTokens;

  // Determine if interactions should be disabled
  const isDisabled = isLoading || !max_tokens_enabled;

  const handleValueChange = (values: number[]) => {
    setMaxTokens(values[0]);
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <Label className="text-sm font-medium">Max Tokens</Label>
        <div className="flex items-center gap-4">
          <span className={`text-sm text-muted-foreground ${isDisabled ? 'opacity-50' : ''}`}>
            {currentValue}
          </span>
          <Switch
            id="token-slider-toggle"
            checked={max_tokens_enabled}
            onCheckedChange={setMaxTokensEnabled}
            disabled={isLoading}
          />
        </div>
      </div>
      <Slider
        defaultValue={[currentValue]}
        max={maxTokens}
        min={minTokens}
        step={1}
        onValueChange={handleValueChange}
        className={`[&_[role=slider]]:h-4 [&_[role=slider]]:w-4 ${isDisabled ? 'opacity-50' : ''}`}
        aria-label="Max tokens"
        disabled={isDisabled}
      />
    </div>
  );
}
