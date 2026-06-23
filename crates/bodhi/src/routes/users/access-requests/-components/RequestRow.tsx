import { UserAccessRequest } from '@bodhiapp/ts-client';

import { LinkRow, ShellIcon } from '@/components/shell';
import { RoleSelect } from '@/routes/users/access-requests/-components/RoleSelect';
import { StatusChip } from '@/routes/users/access-requests/-components/StatusChip';
import { avatarColor, initials, RoleOption, whenText } from '@/routes/users/access-requests/-components/utils';

export interface RequestRowProps {
  request: UserAccessRequest;
  active: boolean;
  roles: RoleOption[];
  selectedRole: string;
  onRole: (role: string) => void;
  onSelect: () => void;
  onApprove: (req: UserAccessRequest) => void;
  onReject: (req: UserAccessRequest) => void;
  disabled: boolean;
}

export function RequestRow({
  request,
  active,
  roles,
  selectedRole,
  onRole,
  onSelect,
  onApprove,
  onReject,
  disabled,
}: RequestRowProps) {
  const pending = request.status === 'pending';
  return (
    <div
      className={`l-listrow ua-row${active ? ' active' : ''}`}
      onClick={onSelect}
      role="option"
      aria-selected={active}
      data-testid={`request-row-${request.username}`}
    >
      <LinkRow onActivate={onSelect} label={`Open access request from ${request.username}`} />
      <div className="ua-icon">
        <div className="ua-avatar" style={{ background: avatarColor(request.username) }}>
          {initials(request.username)}
        </div>
      </div>
      <div className="ua-id">
        <div className="ua-email" data-testid="request-username">
          {request.username}
        </div>
        <div className="ua-sub" data-testid="request-date">
          {whenText(request)}
        </div>
      </div>
      <div className="ua-status-cell">
        <StatusChip status={request.status} />
      </div>
      <div className="ua-role-cell">
        {pending ? (
          <RoleSelect value={selectedRole} roles={roles} onChange={onRole} testId={`role-select-${request.username}`} />
        ) : request.reviewer ? (
          <span className="ua-role-static" data-testid="request-reviewer">
            <ShellIcon name="shield" size={11} /> {request.reviewer}
          </span>
        ) : null}
      </div>
      <div className="ua-act" onClick={(e) => e.stopPropagation()}>
        {pending && (
          <>
            <button
              className="ua-approve"
              onClick={() => onApprove(request)}
              disabled={disabled}
              data-testid={`approve-btn-${request.username}`}
            >
              <ShellIcon name="check" size={13} /> Approve
            </button>
            <button
              className="ua-reject"
              onClick={() => onReject(request)}
              disabled={disabled}
              data-testid={`reject-btn-${request.username}`}
            >
              Reject
            </button>
          </>
        )}
      </div>
    </div>
  );
}
