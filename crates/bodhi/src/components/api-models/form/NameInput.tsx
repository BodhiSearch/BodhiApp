import { forwardRef } from 'react';

import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

interface NameInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  required?: boolean;
  error?: string;
  helpText?: string;
  'data-testid'?: string;
}

export const NameInput = forwardRef<HTMLInputElement, NameInputProps>(
  (
    {
      label = 'Name',
      required = true,
      error,
      helpText,
      className = '',
      'data-testid': testId = 'api-model-name-input',
      ...props
    },
    ref
  ) => {
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
          type="text"
          maxLength={255}
          placeholder="A descriptive name for this API model"
          className={className}
          autoComplete="off"
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
      </div>
    );
  }
);

NameInput.displayName = 'NameInput';
