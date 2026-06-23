import { SettingInfo } from '@bodhiapp/ts-client';

import { LinkRow, ShellIcon } from '@/components/shell';
import { EDITABLE_KEYS, highlight, isModified, sourceBadgeClass } from '@/routes/settings/-components/settingsFormat';

export interface SettingRowProps {
  setting: SettingInfo;
  description?: string;
  active: boolean;
  query: string;
  onSelect: () => void;
}

export function SettingRow({ setting, description, active, query, onSelect }: SettingRowProps) {
  const editable = EDITABLE_KEYS.has(setting.key);
  const atDefault = !isModified(setting);
  const hideValue = setting.source === 'system';
  const valueText = hideValue ? '—' : String(setting.current_value ?? '—');

  return (
    <div
      className={`setting-row${isModified(setting) ? ' modified' : ''}${active ? ' active' : ''}`}
      onClick={onSelect}
      data-testid={`setting-row-${setting.key}`}
    >
      <LinkRow onActivate={onSelect} label={`Open setting ${setting.key}`} />
      <div className="row-key">
        <span className="key-name" data-testid={`setting-key-${setting.key}`}>
          {highlight(setting.key, query)}
        </span>
      </div>
      <div className={`row-value${atDefault ? ' at-default' : ''}`} data-testid={`setting-value-${setting.key}`}>
        {valueText}
      </div>
      <div className="row-actions">
        <span className={`s-badge ${sourceBadgeClass(setting.source)}`} data-testid={`setting-source-${setting.key}`}>
          {setting.source}
        </span>
        <span className="type-badge">{setting.metadata.type}</span>
        {editable && (
          <button
            className="row-edit-btn"
            onClick={(e) => {
              e.stopPropagation();
              onSelect();
            }}
            data-testid={`setting-edit-${setting.key}`}
            aria-label={`Edit ${setting.key}`}
          >
            <ShellIcon name="pencil" size={12} />
          </button>
        )}
      </div>
      {description && <div className="row-desc">{description}</div>}
    </div>
  );
}
