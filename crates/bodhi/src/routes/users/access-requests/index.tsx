import { useCallback, useMemo, useState } from 'react';

import { UserAccessRequest } from '@bodhiapp/ts-client';
import { createFileRoute } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';
import {
  LinkRow,
  ShellFilterTabs,
  ShellIcon,
  ShellPagination,
  useCollapsibleSearch,
  useListKeyNav,
  useShellChrome,
} from '@/components/shell';
import { Skeleton } from '@/components/ui/skeleton';
import '@/components/shell/api-keys.css';
import '@/components/shell/list.css';
import '@/components/shell/user-access-requests.css';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useApproveRequest, useGetAuthenticatedUser, useListAllRequests, useRejectRequest } from '@/hooks/users';
import { useViewTransition } from '@/hooks/useViewTransition';
import { getAvailableRoles } from '@/lib/roles';

export const Route = createFileRoute('/users/access-requests/')({
  component: AccessRequestsPage,
});

const REQUEST_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Users', href: '/users/access-requests/' },
  { label: 'User Access Requests', current: true },
];

type RequestFilter = 'all' | 'pending' | 'approved' | 'rejected';

const REQUEST_FILTER_TABS: { id: RequestFilter; label: string }[] = [
  { id: 'pending', label: 'Pending' },
  { id: 'approved', label: 'Approved' },
  { id: 'rejected', label: 'Rejected' },
  { id: 'all', label: 'All' },
];

const STATUS_ICON: Record<string, string> = {
  pending: 'clock',
  approved: 'check-circle-2',
  rejected: 'x-circle',
};

const fmtDate = (iso: string) =>
  new Date(iso).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });

const whenText = (req: UserAccessRequest) =>
  req.status === 'pending'
    ? `Requested ${fmtDate(req.created_at)}`
    : `${req.status === 'rejected' ? 'Rejected' : 'Approved'} ${fmtDate(req.updated_at)}`;

const AVATAR_COLORS = ['#3E4AA8', '#0F6F67', '#B02A52', '#2F7D1F', '#5E6AD2', '#9A5B12'];
function avatarColor(seed: string) {
  let h = 0;
  for (let i = 0; i < seed.length; i++) h = (h * 31 + seed.charCodeAt(i)) >>> 0;
  return AVATAR_COLORS[h % AVATAR_COLORS.length];
}
function initials(name: string) {
  const local = (name.split('@')[0] || name).replace(/[^a-zA-Z0-9]/g, '');
  return (local.slice(0, 2) || name.slice(0, 2)).toUpperCase();
}

function StatusChip({ status }: { status: string }) {
  return (
    <span className={`ua-status ${status}`} data-testid={`request-status-${status}`}>
      <ShellIcon name={STATUS_ICON[status] ?? 'circle'} size={11} />
      {status.charAt(0).toUpperCase() + status.slice(1)}
    </span>
  );
}

function AccessRequestsContent() {
  useListKeyNav();
  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);
  const [filter, setFilter] = useState<RequestFilter>('pending');
  const [search, setSearch] = useState('');
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [selectedRole, setSelectedRole] = useState<string>('resource_user');

  const { showSuccess, showError } = useToastMessages();
  const { data: userInfo } = useGetAuthenticatedUser();
  const { data: requestsData, isLoading } = useListAllRequests(page, pageSize);

  const userRole = typeof userInfo?.role === 'string' ? userInfo.role : '';
  const availableRoles = useMemo(() => getAvailableRoles(userRole), [userRole]);

  const { mutate: approveRequest, isPending: isApproving } = useApproveRequest({
    onSuccess: () => showSuccess('Request Approved', 'Access granted'),
    onError: (message) => showError('Approval Failed', message),
  });
  const { mutate: rejectRequest, isPending: isRejecting } = useRejectRequest({
    onSuccess: () => showSuccess('Request Rejected', 'Access rejected'),
    onError: (message) => showError('Rejection Failed', message),
  });

  const onApprove = useCallback(
    (req: UserAccessRequest) => approveRequest({ id: req.id, role: selectedRole }),
    [approveRequest, selectedRole]
  );
  const onReject = useCallback((req: UserAccessRequest) => rejectRequest(req.id), [rejectRequest]);

  const withViewTransition = useViewTransition();
  const selectRequest = useCallback(
    (id: string | null) => withViewTransition(() => setSelectedId(id)),
    [withViewTransition]
  );

  const requests = useMemo(() => requestsData?.requests ?? [], [requestsData]);
  const total = requestsData?.total ?? 0;

  const counts = useMemo(() => {
    const c = { all: requests.length, pending: 0, approved: 0, rejected: 0 };
    for (const r of requests) {
      if (r.status === 'pending') c.pending++;
      else if (r.status === 'approved') c.approved++;
      else if (r.status === 'rejected') c.rejected++;
    }
    return c;
  }, [requests]);

  const filterTabs = useMemo(() => REQUEST_FILTER_TABS.map((t) => ({ ...t, count: counts[t.id] })), [counts]);

  const searchNode = useCollapsibleSearch({
    value: search,
    onChange: setSearch,
    placeholder: 'Search requests by email…',
    toggleTestId: 'requests-search-toggle',
    closeTestId: 'requests-search-close',
  });

  const visible = useMemo(() => {
    const q = search.trim().toLowerCase();
    return requests.filter((r) => {
      if (filter !== 'all' && r.status !== filter) return false;
      if (!q) return true;
      return r.username.toLowerCase().includes(q);
    });
  }, [requests, filter, search]);

  const selected = useMemo(() => requests.find((r) => r.id === selectedId) ?? null, [requests, selectedId]);

  const headerActions = useMemo(
    () =>
      counts.pending > 0 ? (
        <span className="tag tag-saffron" data-testid="pending-pill">
          <ShellIcon name="clock" size={12} />
          {counts.pending} pending review
        </span>
      ) : null,
    [counts.pending]
  );

  const railHeader = useMemo(
    () => (selected ? <RequestRailHeader req={selected} onClose={() => selectRequest(null)} /> : null),
    [selected, selectRequest]
  );
  const rail = useMemo(
    () =>
      selected ? (
        <RequestDetailPanel
          req={selected}
          roles={availableRoles}
          selectedRole={selectedRole}
          onRole={setSelectedRole}
          onApprove={onApprove}
          onReject={onReject}
          disabled={isApproving || isRejecting}
        />
      ) : null,
    [selected, availableRoles, selectedRole, onApprove, onReject, isApproving, isRejecting]
  );

  useShellChrome({
    breadcrumb: REQUEST_BREADCRUMB,
    headerActions,
    rail,
    railHeader,
    railDefaultOpen: false,
  });

  return (
    <div
      className="api-keys-screen l-page"
      data-testid="all-requests-page"
      data-pagestatus={isLoading ? 'loading' : 'ready'}
    >
      <div className="l-controls">
        {searchNode.row}
        <div className="l-toolbar">
          <ShellFilterTabs
            tabs={filterTabs}
            value={filter}
            onChange={setFilter}
            label="Filter access requests"
            testIdPrefix="requests-filter"
            loading={isLoading}
          />
          <div className="l-tb-actions">{searchNode.toggle}</div>
        </div>
      </div>

      <div className="l-scroll" data-testid="requests-table">
        {isLoading ? (
          <div style={{ padding: 16 }} data-testid="loading-skeleton">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-12 w-full mb-3" />
            ))}
          </div>
        ) : visible.length === 0 ? (
          <div className="empty-state" data-testid="no-requests">
            <div className="empty-icon">
              <ShellIcon name="user-check" size={28} />
            </div>
            <div className="empty-title">No Access Requests</div>
            <div className="empty-sub">
              {search ? 'No requests match your search.' : 'No access requests match this filter.'}
            </div>
          </div>
        ) : (
          <div className="l-listview">
            <div className="l-listhead">
              <div className="ua-icon" />
              <div className="l-lh ua-id">User</div>
              <div className="l-lh ua-status-cell">Status</div>
              <div className="l-lh ua-role-cell">Role</div>
              <div className="ua-act" />
            </div>
            {visible.map((request) => (
              <RequestRow
                key={request.id}
                request={request}
                active={request.id === selectedId}
                roles={availableRoles}
                selectedRole={selectedRole}
                onRole={setSelectedRole}
                onSelect={() => selectRequest(request.id)}
                onApprove={onApprove}
                onReject={onReject}
                disabled={isApproving || isRejecting}
              />
            ))}
          </div>
        )}
        {total > pageSize && <ShellPagination minimal total={total} page={page} onPage={setPage} pageSize={pageSize} />}
      </div>
    </div>
  );
}

interface RoleOption {
  value: string;
  label: string;
}

function RoleSelect({
  value,
  roles,
  onChange,
  className = 'ua-role-select',
  testId,
}: {
  value: string;
  roles: RoleOption[];
  onChange: (role: string) => void;
  className?: string;
  testId?: string;
}) {
  return (
    <select
      className={className}
      value={value}
      data-testid={testId}
      onClick={(e) => e.stopPropagation()}
      onChange={(e) => {
        e.stopPropagation();
        onChange(e.target.value);
      }}
    >
      {roles.map((r) => (
        <option key={r.value} value={r.value}>
          {r.label}
        </option>
      ))}
    </select>
  );
}

interface RequestRowProps {
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

function RequestRow({
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

function RequestRailHeader({ req, onClose }: { req: UserAccessRequest; onClose: () => void }) {
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: avatarColor(req.username) }}>
        {initials(req.username)}
      </div>
      <div className="dp-head-body">
        <div className="dp-head-title">{req.username}</div>
        <div className="dp-head-sub">User access request</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close" data-testid="request-detail-close">
        <ShellIcon name="x" size={15} />
      </button>
    </div>
  );
}

function RequestDetailPanel({
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

export default function AccessRequestsPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true} minRole="manager">
      <AccessRequestsContent />
    </AppInitializer>
  );
}
