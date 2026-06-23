import { ShellIcon } from '@/components/shell';
import { GroupMeta } from '@/routes/settings/-components/settingsTypes';

export interface SettingsGroupNavProps {
  groups: GroupMeta[];
  counts: Record<string, number>;
  active: string;
  onNavigate: (id: string) => void;
}

export function SettingsGroupNav({ groups, counts, active, onNavigate }: SettingsGroupNavProps) {
  return (
    <div className="settings-screen-nav" data-testid="settings-group-nav">
      <div className="snav-label">Settings Groups</div>
      {groups.map((g) => (
        <button
          key={g.id}
          className={`snav-item${active === g.id ? ' active' : ''}`}
          onClick={() => onNavigate(g.id)}
          data-testid={`settings-group-${g.id}`}
        >
          <ShellIcon name={g.icon} size={13} />
          {g.name}
          <span className="snav-count">{counts[g.id] || 0}</span>
        </button>
      ))}
      <div className="snav-legend">
        <div className="snav-legend-title">Legend</div>
        <div className="snav-legend-rows">
          <div className="snav-legend-row">
            <span className="s-badge s-badge-default">default</span> At default value
          </div>
          <div className="snav-legend-row">
            <span className="s-badge s-badge-modified">modified</span> Overridden
          </div>
          <div className="snav-legend-row">
            <span className="s-badge s-badge-env">env</span> From env var
          </div>
          <div className="snav-legend-row">
            <span className="s-badge s-badge-cmdline">cmd</span> Command line
          </div>
        </div>
      </div>
    </div>
  );
}
