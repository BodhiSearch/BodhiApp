import { UserInfo } from '@bodhiapp/ts-client';

import { LinkRow } from '@/components/shell';
import { Badge } from '@/components/ui/badge';
import { getRoleBadgeVariant, getRoleLabel } from '@/lib/roles';

import { avatarColor, initials } from './usersUtils';

export interface UserRowProps {
  user: UserInfo;
  active: boolean;
  self: boolean;
  onSelect: () => void;
}

export function UserRow({ user, active, self, onSelect }: UserRowProps) {
  return (
    <div
      className={`l-listrow mu-row${active ? ' active' : ''}`}
      onClick={onSelect}
      role="option"
      aria-selected={active}
      data-testid={`user-row-${user.username}`}
    >
      <LinkRow onActivate={onSelect} label={`Open user ${user.username}`} />
      <div className="mu-icon">
        <div className="mu-avatar" style={{ background: avatarColor(user.username) }}>
          {initials(user.username)}
        </div>
      </div>
      <div className="mu-id">
        <div className="mu-username" data-testid="user-username">
          <span data-testid={`user-username-${user.username}`}>{user.username}</span>
        </div>
      </div>
      <div className="mu-role-cell" data-testid={`user-role-${user.username}`}>
        <Badge variant={getRoleBadgeVariant(user.role ?? '')} data-testid="user-role">
          {getRoleLabel(user.role ?? '')}
        </Badge>
        {self && <span className="mu-you-label">You</span>}
      </div>
    </div>
  );
}
