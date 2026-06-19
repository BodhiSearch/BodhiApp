import { RequiredMark } from '@/components/api-models/form/FormSection';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { API_FORMAT_PRESETS, type ApiFormatPreset } from '@/schemas/apiModel';

const formatDisplayName = (format: string): string => {
  const preset = API_FORMAT_PRESETS[format as ApiFormatPreset];
  return preset?.name || format.toUpperCase();
};

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
      <Label htmlFor={testId} className="flex items-center gap-1">
        {label}
        {required && <RequiredMark />}
      </Label>

      <Select value={value} onValueChange={onValueChange} disabled={disabled}>
        <SelectTrigger id={testId} data-testid={testId}>
          <SelectValue placeholder="Select an API format" />
        </SelectTrigger>
        <SelectContent>
          {options.map((format) => (
            <SelectItem key={format} value={format}>
              {formatDisplayName(format)}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      {disabled && (
        <p className="text-sm text-muted-foreground" data-testid={`${testId}-locked-hint`}>
          Format cannot be changed after creation. Delete and recreate the alias to use a different format.
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
