'use client';

import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

interface PrefixInputProps {
  value?: string;
  onChange: (value: string) => void;
  enabled: boolean;
  onEnabledChange: (enabled: boolean) => void;
  error?: string;
  label?: string;
  enabledLabel?: string;
  placeholder?: string;
  helpText?: string;
  'data-testid'?: string;
}

export function PrefixInput({
  value = '',
  onChange,
  enabled,
  onEnabledChange,
  error,
  label = 'Model Prefix',
  enabledLabel = 'Enable prefix',
  placeholder = "e.g., 'azure/', 'openai:', 'my.custom_'",
  helpText = 'Add a prefix to all model names (useful for organization or API routing)',
  'data-testid': testId = 'prefix-input',
}: PrefixInputProps) {
  return (
    <div className="space-y-2">
      <Label htmlFor={testId}>{label}</Label>

      <div className="flex items-center space-x-2">
        <input
          type="checkbox"
          id={`${testId}-enabled`}
          checked={enabled}
          onChange={(e) => onEnabledChange(e.target.checked)}
          data-testid={`${testId}-checkbox`}
          className="rounded border-gray-300 focus:ring-2 focus:ring-blue-500"
        />
        <Label
          htmlFor={`${testId}-enabled`}
          className="text-sm text-muted-foreground cursor-pointer flex-shrink-0"
        >
          {enabledLabel}
        </Label>
        <Input
          id={testId}
          data-testid={testId}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          disabled={!enabled}
          className="flex-1"
        />
      </div>

      {error && (
        <p className="text-sm text-destructive" data-testid={`${testId}-error`}>
          {error}
        </p>
      )}

      {!error && helpText && (
        <p className="text-xs text-muted-foreground" data-testid={`${testId}-help`}>
          {helpText}
        </p>
      )}

      {enabled && value && (
        <div className="text-xs text-muted-foreground">
          <span className="font-medium">Example:</span> {value}gpt-4
        </div>
      )}
    </div>
  );
}