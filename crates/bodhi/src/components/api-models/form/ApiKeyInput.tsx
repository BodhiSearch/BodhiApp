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
          <Label htmlFor={testId} className="flex items-center gap-2">
            {label}
            {required && <Badge variant="secondary">Required</Badge>}
          </Label>
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

        <div className="relative">
          <Input
            {...props}
            ref={ref}
            id={testId}
            data-testid={testId}
            type={showApiKey ? 'text' : 'password'}
            placeholder={getPlaceholder()}
            className={`pr-10 ${className}`}
            autoComplete="off"
          />
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="absolute right-0 top-0 h-full px-3"
            onClick={() => setShowApiKey(!showApiKey)}
            data-testid={`${testId}-visibility-toggle`}
            tabIndex={-1}
          >
            {showApiKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            <span className="sr-only">{showApiKey ? 'Hide API key' : 'Show API key'}</span>
          </Button>
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
