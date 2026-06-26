import type { McpAuthConfigResponse } from '@bodhiapp/ts-client';

import { authConfigTypeLabel } from '@/lib/mcpUtils';

/**
 * The auth mechanisms a user can pick to create an instance against a server: every configured
 * auth-config (header/oauth) PLUS the always-available synthetic `public` (no DB row). The `id` is
 * what the New-Instance deep-link carries as `?auth=` (`public` is the sentinel for no-auth).
 *
 * Shared by the My-MCPs / Explore rail "Connect with" list and (conceptually) the New-Instance form's
 * auth dropdown, so the two stay in lockstep. Ordering mirrors the New-Instance dropdown:
 * explicit mechanisms first, Public LAST.
 */
export interface AuthMechanism {
  id: string;
  label: string;
  type: 'public' | 'header' | 'oauth';
  detail?: string;
}

export const PUBLIC_MECHANISM: AuthMechanism = {
  id: 'public',
  label: 'Public',
  type: 'public',
  detail: 'Always available · no setup',
};

export function buildAuthMechanisms(authConfigs: McpAuthConfigResponse[] | undefined): AuthMechanism[] {
  const explicit: AuthMechanism[] = (authConfigs ?? []).map((c) => ({
    id: c.id,
    label: c.name,
    type: c.type,
    detail: authConfigTypeLabel(c.type),
  }));
  return [...explicit, PUBLIC_MECHANISM];
}
