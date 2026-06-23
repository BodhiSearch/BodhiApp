import { ReactNode } from 'react';

import { Label } from '@/components/ui/label';

interface EnableableInputWrapperProps {
  testId: string;
  enabled: boolean;
  onEnabledChange: (enabled: boolean) => void;
  enabledLabel: string;
  children: ReactNode;
}

export function EnableableInputWrapper({
  testId,
  enabled,
  onEnabledChange,
  enabledLabel,
  children,
}: EnableableInputWrapperProps) {
  return (
    <div className="flex items-center space-x-2">
      <input
        type="checkbox"
        id={`${testId}-enabled`}
        checked={enabled}
        onChange={(e) => onEnabledChange(e.target.checked)}
        data-testid={`${testId}-checkbox`}
        className="rounded border-gray-300 focus:ring-2 focus:ring-blue-500"
      />
      <Label htmlFor={`${testId}-enabled`} className="text-sm text-muted-foreground cursor-pointer flex-shrink-0">
        {enabledLabel}
      </Label>
      {children}
    </div>
  );
}
