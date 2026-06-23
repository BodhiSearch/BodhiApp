import { EnableableInputWrapper } from '@/components/api-models/form/EnableableInputWrapper';
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

      <EnableableInputWrapper
        testId={testId}
        enabled={enabled}
        onEnabledChange={onEnabledChange}
        enabledLabel={enabledLabel}
      >
        <Input
          id={testId}
          data-testid={testId}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          disabled={!enabled}
          className="flex-1"
        />
      </EnableableInputWrapper>

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
