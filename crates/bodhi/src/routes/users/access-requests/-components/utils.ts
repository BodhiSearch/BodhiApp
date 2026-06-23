import { UserAccessRequest } from '@bodhiapp/ts-client';

export type RequestFilter = 'all' | 'pending' | 'approved' | 'rejected';

export interface RoleOption {
  value: string;
  label: string;
}

export const REQUEST_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Users', href: '/users/access-requests/' },
  { label: 'User Access Requests', current: true },
];

export const REQUEST_FILTER_TABS: { id: RequestFilter; label: string }[] = [
  { id: 'pending', label: 'Pending' },
  { id: 'approved', label: 'Approved' },
  { id: 'rejected', label: 'Rejected' },
  { id: 'all', label: 'All' },
];

export const STATUS_ICON: Record<string, string> = {
  pending: 'clock',
  approved: 'check-circle-2',
  rejected: 'x-circle',
};

export const fmtDate = (iso: string) =>
  new Date(iso).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });

export const whenText = (req: UserAccessRequest) =>
  req.status === 'pending'
    ? `Requested ${fmtDate(req.created_at)}`
    : `${req.status === 'rejected' ? 'Rejected' : 'Approved'} ${fmtDate(req.updated_at)}`;

const AVATAR_COLORS = ['#3E4AA8', '#0F6F67', '#B02A52', '#2F7D1F', '#5E6AD2', '#9A5B12'];

export function avatarColor(seed: string) {
  let h = 0;
  for (let i = 0; i < seed.length; i++) h = (h * 31 + seed.charCodeAt(i)) >>> 0;
  return AVATAR_COLORS[h % AVATAR_COLORS.length];
}

export function initials(name: string) {
  const local = (name.split('@')[0] || name).replace(/[^a-zA-Z0-9]/g, '');
  return (local.slice(0, 2) || name.slice(0, 2)).toUpperCase();
}
