'use client';

import { forwardRef } from 'react';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';

interface BaseUrlInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  required?: boolean;
  error?: string;
  helpText?: string;
  showWhen?: boolean;
  'data-testid'?: string;
}

export const BaseUrlInput = forwardRef<HTMLInputElement, BaseUrlInputProps>(
  (
    {
      label = 'Base URL',
      required = true,
      error,
      helpText,
      showWhen = true,
      className = '',
      'data-testid': testId = 'base-url-input',
      ...props
    },
    ref
  ) => {
    if (!showWhen) {
      return null;
    }

    return (
      <div className="space-y-2">
        <Label htmlFor={testId} className="flex items-center gap-2">
          {label}
          {required && <Badge variant="secondary">Required</Badge>}
        </Label>

        <Input
          {...props}
          ref={ref}
          id={testId}
          data-testid={testId}
          type="url"
          placeholder="https://api.your-provider.com/v1"
          className={className}
          autoComplete="url"
        />

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

        {!error && !helpText && (
          <p className="text-xs text-muted-foreground">
            Enter the complete API endpoint URL for your provider
          </p>
        )}
      </div>
    );
  }
);

BaseUrlInput.displayName = 'BaseUrlInput';