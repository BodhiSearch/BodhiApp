import React from 'react';

import { Badge } from '@/components/ui/badge';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';

interface ExtrasSectionProps {
  extraHeaders: string;
  extraBody: string;
  onExtraHeadersChange: (value: string) => void;
  onExtraBodyChange: (value: string) => void;
  extraHeadersError?: string;
  extraBodyError?: string;
}

export function ExtrasSection({
  extraHeaders,
  extraBody,
  onExtraHeadersChange,
  onExtraBodyChange,
  extraHeadersError,
  extraBodyError,
}: ExtrasSectionProps) {
  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <Label htmlFor="extra-headers-input" className="flex items-center gap-2">
          Extra Headers
          <Badge variant="secondary">Optional</Badge>
        </Label>
        <Textarea
          id="extra-headers-input"
          data-testid="extra-headers-input"
          placeholder={'{\n  "anthropic-version": "2023-06-01"\n}'}
          value={extraHeaders}
          onChange={(e) => onExtraHeadersChange(e.target.value)}
          rows={6}
          className="font-mono text-sm"
        />
        {extraHeadersError && (
          <p className="text-sm text-destructive" data-testid="extra-headers-input-error">
            {extraHeadersError}
          </p>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="extra-body-input" className="flex items-center gap-2">
          Extra Body
          <Badge variant="secondary">Optional</Badge>
        </Label>
        <Textarea
          id="extra-body-input"
          data-testid="extra-body-input"
          placeholder={'{\n  "max_tokens": 4096\n}'}
          value={extraBody}
          onChange={(e) => onExtraBodyChange(e.target.value)}
          rows={6}
          className="font-mono text-sm"
        />
        {extraBodyError && (
          <p className="text-sm text-destructive" data-testid="extra-body-input-error">
            {extraBodyError}
          </p>
        )}
      </div>
    </div>
  );
}
