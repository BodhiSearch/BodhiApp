import { UserInfo } from '@bodhiapp/ts-client';

import { ShellIcon } from '@/components/shell';
import { getRoleLabel } from '@/lib/roles';

import { avatarColor, initials } from './usersUtils';

export function UserRailHeader({ user, self, onClose }: { user: UserInfo; self: boolean; onClose: () => void }) {
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: avatarColor(user.username) }}>
        {initials(user.username)}
      </div>
      <div className="dp-head-body">
        <div className="dp-head-title mono">{user.username}</div>
        <div className="dp-head-sub">
          {getRoleLabel(user.role ?? '')}
          {self ? ' · You' : ''}
        </div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close" data-testid="user-detail-close">
        <ShellIcon name="x" size={15} />
      </button>
    </div>
  );
}
