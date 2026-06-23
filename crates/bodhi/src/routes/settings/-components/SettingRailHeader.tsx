import { SettingInfo } from '@bodhiapp/ts-client';

import { ShellIcon } from '@/components/shell';

export function SettingRailHeader({
  setting,
  groupName,
  onClose,
}: {
  setting: SettingInfo;
  groupName: string;
  onClose: () => void;
}) {
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: 'var(--c-indigo-bg)', color: 'var(--c-indigo-text)' }}>
        <ShellIcon name="sliders-horizontal" size={15} />
      </div>
      <div className="dp-head-body">
        <div className="dp-head-title mono">{setting.key}</div>
        <div className="dp-head-sub">{groupName}</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close" data-testid="setting-detail-close">
        <ShellIcon name="x" size={15} />
      </button>
    </div>
  );
}
