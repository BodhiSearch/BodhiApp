import type { McpAuthType as CatalogAuthType } from '@bodhiapp/reference-api-types';

import { ShellIcon } from '@/components/shell';
import './auth-badges.css';

/**
 * Auth-type visual vocabulary, shared by the Explore list/rail (reference-catalog `auth_type`) and the
 * rail's connect-with mechanisms (local auth-config `type`). One badge per kind: OAuth → lock/indigo,
 * API Key → key/saffron, Public → unlock/leaf. The reference API's `oauth-dcr` / `oauth-pre-registered`
 * both collapse to the OAuth badge; the placeholder `http` falls back to a neutral chip.
 */

export type AuthKind = 'oauth' | 'key' | 'public' | 'http';

interface AuthMeta {
  icon: string;
  label: string;
  cls: string;
}

const AUTH_META: Record<AuthKind, AuthMeta> = {
  oauth: { icon: 'lock', label: 'OAuth', cls: 'auth-badge-oauth' },
  key: { icon: 'key', label: 'API Key', cls: 'auth-badge-key' },
  public: { icon: 'unlock', label: 'Public', cls: 'auth-badge-public' },
  http: { icon: 'shield', label: 'HTTP', cls: 'auth-badge-http' },
};

/** Normalize any catalog `auth_type` or local auth-config `type` to a visual kind. */
export function authKind(type: CatalogAuthType | 'oauth' | 'header' | 'public' | string): AuthKind {
  switch (type) {
    case 'oauth':
    case 'oauth-dcr':
    case 'oauth-pre-registered':
      return 'oauth';
    case 'key':
    case 'header':
      return 'key';
    case 'public':
      return 'public';
    default:
      return 'http';
  }
}

export function AuthBadge({ type }: { type: CatalogAuthType | string }) {
  const meta = AUTH_META[authKind(type)];
  return (
    <span className={`auth-badge ${meta.cls}`}>
      <ShellIcon name={meta.icon} size={10} />
      {meta.label}
    </span>
  );
}
