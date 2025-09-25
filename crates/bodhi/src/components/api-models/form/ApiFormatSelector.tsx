'use client';

import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Badge } from '@/components/ui/badge';

interface ApiFormatSelectorProps {
  value?: string;
  onValueChange: (value: string) => void;
  options?: string[];
  label?: string;
  required?: boolean;
  disabled?: boolean;
  error?: string;
  'data-testid'?: string;
}

export function ApiFormatSelector({
  value,
  onValueChange,
  options = ['openai'],
  label = 'API Format',
  required = true,
  disabled = false,
  error,
  'data-testid': testId = 'api-format-selector',
}: ApiFormatSelectorProps) {
  return (
    <div className="space-y-2">
      <Label htmlFor={testId} className="flex items-center gap-2">
        {label}
        {required && <Badge variant="secondary">Required</Badge>}
      </Label>

      <Select value={value} onValueChange={onValueChange} disabled={disabled}>
        <SelectTrigger id={testId} data-testid={testId}>
          <SelectValue placeholder="Select an API format" />
        </SelectTrigger>
        <SelectContent>
          {options.map((format) => (
            <SelectItem key={format} value={format}>
              {format.toUpperCase()}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      {error && (
        <p className="text-sm text-destructive" data-testid={`${testId}-error`}>
          {error}
        </p>
      )}

      {!error && options.length === 1 && (
        <p className="text-xs text-muted-foreground">Currently supporting OpenAI-compatible APIs</p>
      )}
    </div>
  );
}
