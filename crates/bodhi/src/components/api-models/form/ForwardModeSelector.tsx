'use client';

import { Label } from '@/components/ui/label';

interface ForwardModeSelectorProps {
  forwardAll: boolean;
  onForwardAllChange: (value: boolean) => void;
  prefixEnabled: boolean;
  prefix?: string;
  error?: string;
  'data-testid'?: string;
}

export function ForwardModeSelector({
  forwardAll,
  onForwardAllChange,
  prefixEnabled,
  prefix,
  error,
  'data-testid': testId = 'forward-mode-selector',
}: ForwardModeSelectorProps) {
  const isForwardAllDisabled = !prefixEnabled || !prefix || prefix.trim() === '';

  return (
    <div className="space-y-3" data-testid={testId}>
      <Label>Request Forwarding Mode</Label>

      <div className="space-y-2">
        <div className="flex items-center space-x-2">
          <input
            type="radio"
            id={`${testId}-forward-all`}
            name={`${testId}-mode`}
            checked={forwardAll}
            onChange={() => onForwardAllChange(true)}
            disabled={isForwardAllDisabled}
            data-testid={`${testId}-forward-all`}
            className="rounded-full border-gray-300 focus:ring-2 focus:ring-blue-500"
          />
          <Label
            htmlFor={`${testId}-forward-all`}
            className={`text-sm cursor-pointer ${isForwardAllDisabled ? 'text-muted-foreground opacity-50' : ''}`}
          >
            Forward all requests with prefix
          </Label>
        </div>

        <div className="flex items-center space-x-2">
          <input
            type="radio"
            id={`${testId}-forward-selected`}
            name={`${testId}-mode`}
            checked={!forwardAll}
            onChange={() => onForwardAllChange(false)}
            data-testid={`${testId}-forward-selected`}
            className="rounded-full border-gray-300 focus:ring-2 focus:ring-blue-500"
          />
          <Label htmlFor={`${testId}-forward-selected`} className="text-sm cursor-pointer">
            Forward for selected models only
          </Label>
        </div>
      </div>

      {isForwardAllDisabled && (
        <p className="text-xs text-muted-foreground" data-testid={`${testId}-help`}>
          Enable prefix and provide a value to use &quot;Forward all requests with prefix&quot; mode
        </p>
      )}

      {error && (
        <p className="text-sm text-destructive" data-testid={`${testId}-error`}>
          {error}
        </p>
      )}
    </div>
  );
}
