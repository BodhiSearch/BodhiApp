import { ReactNode } from 'react';

import { SettingInfo } from '@bodhiapp/ts-client';

/** Backend `EDIT_SETTINGS_ALLOWED` (routes_settings.rs) — the only editable settings. */
export const EDITABLE_KEYS = new Set(['BODHI_EXEC_VARIANT', 'BODHI_KEEP_ALIVE_SECS']);

export const isModified = (s: SettingInfo) => String(s.current_value) !== String(s.default_value);
export const isEnv = (s: SettingInfo) => s.source === 'environment';

export function sourceBadgeClass(source: string): string {
  switch (source) {
    case 'environment':
      return 's-badge-env';
    case 'command_line':
      return 's-badge-cmdline';
    case 'system':
      return 's-badge-system';
    case 'default':
      return 's-badge-default';
    default:
      return 's-badge-modified';
  }
}

export function highlight(text: string, query: string): ReactNode {
  if (!query) return text;
  const idx = text.toLowerCase().indexOf(query);
  if (idx < 0) return text;
  return (
    <>
      {text.slice(0, idx)}
      <mark>{text.slice(idx, idx + query.length)}</mark>
      {text.slice(idx + query.length)}
    </>
  );
}
