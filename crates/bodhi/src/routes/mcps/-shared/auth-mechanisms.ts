import type { McpAuthConfigResponse } from '@bodhiapp/ts-client';

import { authConfigDetail, authConfigTypeLabel } from '@/lib/mcpUtils';
import { type AuthKind, authKind } from '@/routes/mcps/-shared/auth-badges';

/**
 * The auth mechanisms a user can pick to create an instance against a server: every configured
 * auth-config (header/oauth) PLUS the always-available synthetic `public` (no DB row). The `id` is
 * what the New-Instance deep-link carries as `?auth=` (`public` is the sentinel for no-auth).
 *
 * Shared by the My-MCPs / Explore rail "Connect with" list. Ordering mirrors the prototype's
 * `availableAuth`: Public FIRST, then the explicit mechanisms.
 */
export interface AuthMechanism {
  id: string;
  /** Display title — OAuth / API Key / Public. */
  title: string;
  /** Visual kind for the row icon tile + badge. */
  kind: AuthKind;
  /** Subtitle: "Always available · no setup" (public) or "<name> · <detail>" (explicit). */
  detail: string;
}

export const PUBLIC_MECHANISM: AuthMechanism = {
  id: 'public',
  title: 'Public',
  kind: 'public',
  detail: 'Always available · no setup',
};

function titleFor(kind: AuthKind): string {
  if (kind === 'oauth') return 'OAuth';
  if (kind === 'key') return 'API Key';
  return 'Public';
}

export function buildAuthMechanisms(authConfigs: McpAuthConfigResponse[] | undefined): AuthMechanism[] {
  const explicit: AuthMechanism[] = (authConfigs ?? []).map((c) => {
    const kind = authKind(c.type);
    const detailParts = [c.name, c.type === 'oauth' ? authConfigTypeLabel(c.type) : authConfigDetail(c)].filter(
      Boolean
    );
    return { id: c.id, title: titleFor(kind), kind, detail: detailParts.join(' · ') };
  });
  // Public FIRST (mirrors the prototype's `availableAuth` "Connect with" ordering).
  return [PUBLIC_MECHANISM, ...explicit];
}
