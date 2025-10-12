'use client';

import { forwardRef, useState } from 'react';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Eye, EyeOff, ExternalLink } from 'lucide-react';

interface ApiKeyInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  required?: boolean;
  error?: string;
  helpText?: string;
  docUrl?: string;
  mode?: 'create' | 'edit' | 'setup';
  enabled: boolean;
  onEnabledChange: (enabled: boolean) => void;
  enabledLabel?: string;
  'data-testid'?: string;
}

export const ApiKeyInput = forwardRef<HTMLInputElement, ApiKeyInputProps>(
  (
    {
      label = 'API Key',
      required = true,
      error,
      helpText,
      docUrl,
      mode = 'create',
      enabled,
      onEnabledChange,
      enabledLabel = 'Use API key',
      className = '',
      'data-testid': testId = 'api-key-input',
      ...props
    },
    ref
  ) => {
    const [showApiKey, setShowApiKey] = useState(false);

    const getPlaceholder = () => {
      switch (mode) {
        case 'edit':
          return 'Leave empty to keep existing key';
        case 'setup':
          return 'Enter your API key...';
        default:
          return 'Enter your API key';
      }
    };

    const getHelpText = () => {
      if (helpText) return helpText;

      switch (mode) {
        case 'edit':
          return 'Leave this field empty to keep your existing API key';
        case 'setup':
          return 'Your API key will be stored securely and encrypted';
        default:
          return 'Your API key is stored securely';
      }
    };

    return (
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <Label htmlFor={testId}>{label}</Label>
          {docUrl && (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="h-auto p-1 text-xs text-primary hover:text-primary/80"
              asChild
            >
              <a href={docUrl} target="_blank" rel="noopener noreferrer" className="flex items-center gap-1">
                <ExternalLink className="w-3 h-3" />
                Get API Key
              </a>
            </Button>
          )}
        </div>

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
          <div className="relative flex-1">
            <Input
              {...props}
              ref={ref}
              id={testId}
              data-testid={testId}
              type={showApiKey ? 'text' : 'password'}
              placeholder={getPlaceholder()}
              className={`pr-10 ${className}`}
              autoComplete="off"
              disabled={!enabled}
            />
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="absolute right-0 top-0 h-full px-3"
              onClick={() => setShowApiKey(!showApiKey)}
              data-testid={`${testId}-visibility-toggle`}
              tabIndex={-1}
              disabled={!enabled}
            >
              {showApiKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
              <span className="sr-only">{showApiKey ? 'Hide API key' : 'Show API key'}</span>
            </Button>
          </div>
        </div>

        {error && (
          <p className="text-sm text-destructive" data-testid={`${testId}-error`}>
            {error}
          </p>
        )}

        {!error && getHelpText() && (
          <p className="text-xs text-muted-foreground" data-testid={`${testId}-help`}>
            {getHelpText()}
          </p>
        )}
      </div>
    );
  }
);

ApiKeyInput.displayName = 'ApiKeyInput';
