import { UserAccessRequest } from '@bodhiapp/ts-client';

import { ShellIcon } from '@/components/shell';
import { RoleSelect } from '@/routes/users/access-requests/-components/RoleSelect';
import { StatusChip } from '@/routes/users/access-requests/-components/StatusChip';
import { fmtDate, RoleOption, whenText } from '@/routes/users/access-requests/-components/utils';

export function RequestDetailPanel({
  req,
  roles,
  selectedRole,
  onRole,
  onApprove,
  onReject,
  disabled,
}: {
  req: UserAccessRequest;
  roles: RoleOption[];
  selectedRole: string;
  onRole: (role: string) => void;
  onApprove: (req: UserAccessRequest) => void;
  onReject: (req: UserAccessRequest) => void;
  disabled: boolean;
}) {
  const pending = req.status === 'pending';
  const approved = req.status === 'approved';
  return (
    <div className="dp-panel" data-testid="request-detail-rail">
      <div className="dp-status-row">
        <StatusChip status={req.status} />
        <span className="dp-head-sub" style={{ marginLeft: 'auto' }}>
          {whenText(req)}
        </span>
      </div>
      <div className="dp-body">
        <div className="dp-section">
          <div className="dp-sec-lbl">Account</div>
          <div className="dp-rows">
            <div className="dp-row">
              <span className="dp-row-k">
                <ShellIcon name="at-sign" size={13} /> Email
              </span>
              <span className="dp-row-v mono">{req.username}</span>
            </div>
            <div className="dp-row">
              <span className="dp-row-k">
                <ShellIcon name="calendar" size={13} /> Requested
              </span>
              <span className="dp-row-v">{fmtDate(req.created_at)}</span>
            </div>
          </div>
        </div>

        {pending ? (
          <div className="dp-section">
            <div className="dp-sec-lbl">Assign role</div>
            <div className="dp-field">
              <RoleSelect
                value={selectedRole}
                roles={roles}
                onChange={onRole}
                className="ua-role-select dp-role-select"
                testId="request-detail-role-select"
              />
              <span className="dp-field-hint">The role is granted to this user when you approve the request.</span>
            </div>
          </div>
        ) : null}

        <div className="dp-section">
          <div className="dp-sec-lbl">Timeline</div>
          <div className="dp-rows">
            <div className="dp-row">
              <span className="dp-row-k">
                <ShellIcon name="clock" size={13} /> Requested
              </span>
              <span className="dp-row-v">{fmtDate(req.created_at)}</span>
            </div>
            {!pending && (
              <div className="dp-row">
                <span className="dp-row-k">
                  <ShellIcon name={approved ? 'check' : 'x'} size={13} /> {approved ? 'Approved' : 'Rejected'}
                </span>
                <span className="dp-row-v">{fmtDate(req.updated_at)}</span>
              </div>
            )}
          </div>
        </div>
      </div>
      <div className="dp-foot">
        {pending ? (
          <>
            <button
              className="dp-btn dp-btn-approve"
              onClick={() => onApprove(req)}
              disabled={disabled}
              data-testid="request-detail-approve"
            >
              <ShellIcon name="check" size={14} /> Approve
            </button>
            <button
              className="dp-btn dp-btn-danger"
              onClick={() => onReject(req)}
              disabled={disabled}
              data-testid="request-detail-reject"
            >
              <ShellIcon name="x" size={14} /> Reject
            </button>
          </>
        ) : (
          <div className="ua-decided-note">
            <ShellIcon name={approved ? 'check-circle-2' : 'x-circle'} size={14} />
            <span>
              {approved ? 'Approved' : 'Rejected'} {fmtDate(req.updated_at)}
            </span>
          </div>
        )}
      </div>
    </div>
  );
}
