import { useRef, useState } from 'react';

import { SettingInfo } from '@bodhiapp/ts-client';

import { DetailRail, DetailRailBody, DetailRailRows, DetailRailSection } from '@/components/detail-rail';
import { ShellIcon } from '@/components/shell';
import { useDeleteSetting, useUpdateSetting } from '@/hooks/settings';
import { useToastMessages } from '@/hooks/useToastMessages';
import { EDITABLE_KEYS, sourceBadgeClass } from '@/routes/settings/-components/settingsFormat';
import { parseSettingValue, SettingValueInput, settingValueHint } from '@/routes/settings/-shared/settingInput';

export interface SettingRailPanelProps {
  setting: SettingInfo;
  description?: string;
}

export function SettingRailPanel({ setting, description }: SettingRailPanelProps) {
  const editable = EDITABLE_KEYS.has(setting.key);
  const hideValue = setting.source === 'system';
  const { showSuccess, showError } = useToastMessages();

  const [draft, setDraft] = useState<string>(() => String(setting.current_value ?? ''));
  // Reset the draft when a different setting is selected.
  const keyRef = useRef(setting.key);
  if (keyRef.current !== setting.key) {
    keyRef.current = setting.key;
    setDraft(String(setting.current_value ?? ''));
  }

  const updateSetting = useUpdateSetting({
    onSuccess: () => showSuccess('Success', `Setting ${setting.key} updated`),
    onError: (message) => showError('Error', message),
  });
  const deleteSetting = useDeleteSetting({
    onSuccess: () => showSuccess('Reset', `Setting ${setting.key} reset to default`),
    onError: (message) => showError('Error', message),
  });

  const dirty = draft !== String(setting.current_value ?? '');
  const busy = updateSetting.isPending || deleteSetting.isPending;

  const onSave = () => {
    const parsed = parseSettingValue(setting.metadata, draft);
    if (!parsed.ok) {
      showError('Error', parsed.error);
      return;
    }
    updateSetting.mutate({ key: setting.key, value: parsed.value });
  };

  return (
    <DetailRail className="settings-screen-rail" testId={`setting-detail-${setting.key}`}>
      <div className="dp-status-row">
        <span className={`s-badge ${sourceBadgeClass(setting.source)}`}>{setting.source}</span>
        <span className="type-badge">{setting.metadata.type}</span>
      </div>

      <DetailRailBody>
        {description && (
          <DetailRailSection>
            <p className="dp-desc">{description}</p>
          </DetailRailSection>
        )}

        <DetailRailSection label="Values">
          <DetailRailRows>
            {!hideValue && (
              <div className="dp-row">
                <span className="dp-row-k">
                  <ShellIcon name="circle-dot" size={13} /> Current
                </span>
                <span className="dp-row-v mono">{String(setting.current_value ?? '—')}</span>
              </div>
            )}
            <div className="dp-row">
              <span className="dp-row-k">
                <ShellIcon name="rotate-ccw" size={13} /> Default
              </span>
              <span className="dp-row-v mono">{String(setting.default_value ?? '—')}</span>
            </div>
          </DetailRailRows>
        </DetailRailSection>

        {editable ? (
          <DetailRailSection label="New value">
            <div className="dp-field">
              <SettingValueInput setting={setting} value={draft} onChange={setDraft} testId="setting-new-value" />
              <span className="dp-field-hint">{settingValueHint(setting.metadata)}</span>
            </div>
          </DetailRailSection>
        ) : (
          <DetailRailSection>
            <div className="dp-readonly-note" data-testid="setting-readonly-note">
              <ShellIcon name="lock" size={14} />
              <div>This setting is read-only (set via {setting.source}).</div>
            </div>
          </DetailRailSection>
        )}
      </DetailRailBody>

      {editable && (
        <div className="dp-foot">
          <button
            className="dp-btn dp-btn-accent"
            disabled={!dirty || busy}
            onClick={onSave}
            data-testid="setting-save"
          >
            <ShellIcon name="check" size={14} /> {dirty ? 'Save changes' : 'Saved'}
          </button>
          <div className="dp-foot-row">
            <button
              className="dp-btn dp-btn-outline"
              onClick={() => setDraft(String(setting.current_value ?? ''))}
              data-testid="setting-cancel"
            >
              Cancel
            </button>
            {String(setting.current_value) !== String(setting.default_value) && (
              <button
                className="dp-btn dp-btn-outline"
                disabled={busy}
                onClick={() => deleteSetting.mutate({ key: setting.key })}
                data-testid="setting-reset"
              >
                <ShellIcon name="rotate-ccw" size={13} /> Reset
              </button>
            )}
          </div>
        </div>
      )}
    </DetailRail>
  );
}
