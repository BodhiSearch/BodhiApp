import { useMemo, useState } from 'react';

import { UserInfo } from '@bodhiapp/ts-client';

import { DetailRail, DetailRailBody, DetailRailRows, DetailRailSection } from '@/components/detail-rail';
import { ShellIcon } from '@/components/shell';
import { Badge } from '@/components/ui/badge';
import { AuthenticatedUser, useChangeUserRole, useRemoveUser } from '@/hooks/users';
import { useToastMessages } from '@/hooks/useToastMessages';
import { getAvailableRoles, getRoleBadgeVariant, getRoleLabel } from '@/lib/roles';

import { canModify, isSelf } from './usersUtils';

export interface UserRailPanelProps {
  user: UserInfo;
  me?: AuthenticatedUser;
  myRole: string;
}

export function UserRailPanel({ user, me, myRole }: UserRailPanelProps) {
  const self = isSelf(user, me);
  const modifiable = canModify(user, me, myRole);
  const availableRoles = useMemo(() => getAvailableRoles(myRole), [myRole]);
  const { showSuccess, showError } = useToastMessages();

  const [draftRole, setDraftRole] = useState(user.role ?? '');
  const [confirmRemove, setConfirmRemove] = useState(false);

  // Reset when a different user is selected.
  const [trackedId, setTrackedId] = useState(user.user_id);
  if (trackedId !== user.user_id) {
    setTrackedId(user.user_id);
    setDraftRole(user.role ?? '');
    setConfirmRemove(false);
  }

  const changeRole = useChangeUserRole({
    onSuccess: () => showSuccess('Role updated', `${user.username} is now ${getRoleLabel(draftRole)}`),
    onError: (message) => showError('Error', message),
  });
  const removeUser = useRemoveUser({
    onSuccess: () => showSuccess('User removed', `${user.username} was removed`),
    onError: (message) => showError('Error', message),
  });

  const dirty = draftRole !== (user.role ?? '');
  const busy = changeRole.isPending || removeUser.isPending;

  return (
    <DetailRail className="manage-users-rail" testId={`user-detail-${user.username}`}>
      <div className="dp-status-row">
        <Badge variant={getRoleBadgeVariant(user.role ?? '')}>{getRoleLabel(user.role ?? '')}</Badge>
        {self && (
          <span className="mu-you-label" style={{ marginLeft: 'auto' }}>
            This is you
          </span>
        )}
      </div>

      <DetailRailBody>
        <DetailRailSection label="Account">
          <DetailRailRows>
            <div className="dp-row">
              <span className="dp-row-k">
                <ShellIcon name="at-sign" size={13} /> Username
              </span>
              <span className="dp-row-v mono">{user.username}</span>
            </div>
            <div className="dp-row">
              <span className="dp-row-k">
                <ShellIcon name="shield" size={13} /> Current role
              </span>
              <span className="dp-row-v">{getRoleLabel(user.role ?? '')}</span>
            </div>
          </DetailRailRows>
        </DetailRailSection>

        {modifiable ? (
          <DetailRailSection label="Change role">
            <div className="dp-field">
              <select
                className="dp-role-select"
                value={draftRole}
                onChange={(e) => setDraftRole(e.target.value)}
                data-testid={`role-select-${user.username}`}
              >
                {availableRoles.map((r) => (
                  <option key={r.value} value={r.value}>
                    {r.label}
                  </option>
                ))}
              </select>
              <span className="dp-field-hint">Updating the role changes what this user can access across Bodhi.</span>
            </div>
          </DetailRailSection>
        ) : (
          <DetailRailSection>
            <div className="dp-readonly-note" data-testid={`no-actions-${user.username}`}>
              <ShellIcon name={self ? 'user' : 'lock'} size={14} />
              <span data-testid={self ? 'current-user-indicator' : 'restricted-user-indicator'}>
                {self
                  ? "You can't change your own role or remove your own account. Ask another admin if you need changes."
                  : "This user outranks you, so you can't change their role or remove them."}
              </span>
            </div>
          </DetailRailSection>
        )}
      </DetailRailBody>

      {modifiable && (
        <div className="dp-foot">
          <button
            className="dp-btn dp-btn-accent"
            disabled={!dirty || busy}
            onClick={() => changeRole.mutate({ userId: user.user_id, newRole: draftRole })}
            data-testid={`save-role-${user.username}`}
          >
            <ShellIcon name="check" size={14} /> {dirty ? 'Save changes' : 'Saved'}
          </button>
          {confirmRemove ? (
            <button
              className="dp-btn dp-btn-danger"
              disabled={busy}
              onClick={() => removeUser.mutate(user.user_id)}
              data-testid={`remove-user-btn-${user.username}`}
            >
              <ShellIcon name="trash-2" size={14} /> Confirm remove
            </button>
          ) : (
            <button
              className="dp-btn dp-btn-danger"
              onClick={() => setConfirmRemove(true)}
              data-testid={`remove-user-btn-${user.username}`}
            >
              <ShellIcon name="trash-2" size={14} /> Remove user
            </button>
          )}
          {confirmRemove && (
            <div className="dp-field-hint" style={{ textAlign: 'center' }}>
              They&apos;ll lose all access immediately. Click again to confirm.
            </div>
          )}
        </div>
      )}
    </DetailRail>
  );
}
