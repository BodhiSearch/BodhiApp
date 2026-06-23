import { UserInfo } from '@bodhiapp/ts-client';

import { AuthenticatedUser } from '@/hooks/users';
import { getRoleLevel } from '@/lib/roles';

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

export function isSelf(user: UserInfo, me?: AuthenticatedUser) {
  return me?.auth_status === 'logged_in' && user.user_id === me.user_id;
}

/** Mirror of UserActionsCell: self can't be modified; target must not outrank the actor. */
export function canModify(user: UserInfo, me: AuthenticatedUser | undefined, myRole: string) {
  if (isSelf(user, me)) return false;
  return getRoleLevel(user.role ?? '') <= getRoleLevel(myRole);
}
