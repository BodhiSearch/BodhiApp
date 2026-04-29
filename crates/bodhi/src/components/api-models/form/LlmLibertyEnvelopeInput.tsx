import { useState } from 'react';

import { CopyButton } from '@/components/CopyButton';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { validateLlmLibertyEnvelope, type ParsedEnvelopeSummary } from '@/schemas/llmLibertyEnvelope';

const LOGIN_COMMAND = 'npx @bodhiapp/llm-liberty@latest login';

const PLACEHOLDER = `{
  "version": "1.0.0",
  "provider": "anthropic",
  "access_token": "...",
  "refresh_token": "...",
  "expires_at": 1234567890,
  ...
}`;

interface LlmLibertyEnvelopeInputProps {
  value: string;
  onChange: (value: string) => void;
  error?: string;
  mode: 'create' | 'edit' | 'setup';
  hasStoredCredentials?: boolean;
}

export function LlmLibertyEnvelopeInput({
  value,
  onChange,
  error,
  mode,
  hasStoredCredentials,
}: LlmLibertyEnvelopeInputProps) {
  const [parseError, setParseError] = useState<string>('');
  const [summary, setSummary] = useState<ParsedEnvelopeSummary | null>(null);

  const handleChange = (text: string) => {
    onChange(text);
    if (!text.trim()) {
      setParseError('');
      setSummary(null);
      return;
    }
    const result = validateLlmLibertyEnvelope(text);
    if (result.ok) {
      setParseError('');
      setSummary(result.summary);
    } else {
      setParseError(result.error);
      setSummary(null);
    }
  };

  const displayError = error || parseError;
  const isEditMode = mode === 'edit';

  return (
    <div className="space-y-2">
      <Label htmlFor="llm-liberty-envelope">
        LLM Liberty OAuth Credentials
        {isEditMode && hasStoredCredentials && (
          <span className="ml-2 text-sm text-muted-foreground">(leave empty to keep existing credentials)</span>
        )}
      </Label>
      <p className="text-sm text-muted-foreground">
        Run the login command to get credentials, then paste the JSON output below.
      </p>
      <div className="flex items-center gap-2 rounded-md bg-muted px-3 py-2 font-mono text-sm">
        <code className="flex-1 select-all">{LOGIN_COMMAND}</code>
        <CopyButton text={LOGIN_COMMAND} />
      </div>
      <Textarea
        id="llm-liberty-envelope"
        data-testid="llm-liberty-envelope-input"
        value={value}
        onChange={(e) => handleChange(e.target.value)}
        placeholder={PLACEHOLDER}
        rows={8}
        className={`font-mono text-xs ${displayError ? 'border-destructive' : ''}`}
      />
      {displayError && (
        <p className="text-sm text-destructive" data-testid="llm-liberty-envelope-error">
          {displayError}
        </p>
      )}
      {summary && !displayError && (
        <div className="rounded-md bg-muted p-3 text-sm space-y-1" data-testid="llm-liberty-envelope-summary">
          <div className="flex gap-2">
            <span className="font-medium">Provider:</span>
            <span className="capitalize">{summary.provider}</span>
          </div>
          <div className="flex gap-2">
            <span className="font-medium">Expires:</span>
            <span>{summary.expiresAt.toLocaleString()}</span>
          </div>
          <div className="flex gap-2">
            <span className="font-medium">Refresh token:</span>
            <span>{summary.hasRefreshToken ? 'present' : 'absent'}</span>
          </div>
        </div>
      )}
    </div>
  );
}
