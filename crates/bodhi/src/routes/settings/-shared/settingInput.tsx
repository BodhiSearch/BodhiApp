import { SettingInfo, SettingMetadata } from '@bodhiapp/ts-client';

import { Switch } from '@/components/ui/switch';

/**
 * Shared setting-value logic for both the V1 edit dialog and the V2 detail-rail editor, so the
 * parse/validate rules can't diverge. The V1 dialog renders shadcn controls; the V2 rail renders
 * the design's raw `.field-input`/`.field-select` (via {@link SettingValueInput}). Both call
 * {@link parseSettingValue} before mutating.
 */

export type ParsedSetting = { ok: true; value: string | number | boolean } | { ok: false; error: string };

/** Coerce + validate the draft string against the setting's metadata. */
export function parseSettingValue(metadata: SettingMetadata, raw: string): ParsedSetting {
  if (metadata.type === 'number') {
    const value = Number(raw);
    if (isNaN(value)) {
      return { ok: false, error: 'Invalid number' };
    }
    if (value < metadata.min || value > metadata.max) {
      return { ok: false, error: `Value must be between ${metadata.min} and ${metadata.max}` };
    }
    return { ok: true, value };
  }
  if (metadata.type === 'boolean') {
    return { ok: true, value: raw === 'true' };
  }
  return { ok: true, value: raw };
}

interface SettingValueInputProps {
  setting: SettingInfo;
  value: string;
  onChange: (value: string) => void;
  testId?: string;
}

/**
 * The V2-rail type-specific control, styled with the design's raw `.field-*` classes (the V1
 * dialog keeps its shadcn equivalents). Value is always a string; callers coerce via
 * {@link parseSettingValue} on save.
 */
export function SettingValueInput({ setting, value, onChange, testId }: SettingValueInputProps) {
  const { metadata } = setting;

  if (metadata.type === 'option') {
    return (
      <select className="field-select" value={value} onChange={(e) => onChange(e.target.value)} data-testid={testId}>
        {metadata.options.map((option) => (
          <option key={option} value={option}>
            {option}
          </option>
        ))}
      </select>
    );
  }

  if (metadata.type === 'boolean') {
    const checked = value === 'true';
    return (
      <div className="field-switch">
        <Switch checked={checked} onCheckedChange={(c) => onChange(String(c))} data-testid={testId} />
        <span>{checked ? 'Enabled' : 'Disabled'}</span>
      </div>
    );
  }

  if (metadata.type === 'number') {
    return (
      <input
        className="field-input"
        type="number"
        value={value}
        min={metadata.min}
        max={metadata.max}
        onChange={(e) => onChange(e.target.value)}
        placeholder={String(setting.default_value ?? '')}
        data-testid={testId}
      />
    );
  }

  return (
    <input
      className="field-input"
      type="text"
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={String(setting.default_value ?? '')}
      data-testid={testId}
    />
  );
}

/** The field hint shown under the input (matches the prototype copy). */
export function settingValueHint(metadata: SettingMetadata): string {
  switch (metadata.type) {
    case 'number':
      return `Enter a numeric value (${metadata.min}–${metadata.max}).`;
    case 'option':
      return 'Choose from the available options.';
    case 'boolean':
      return 'Toggle the value on or off.';
    default:
      return 'Enter the new string value.';
  }
}
