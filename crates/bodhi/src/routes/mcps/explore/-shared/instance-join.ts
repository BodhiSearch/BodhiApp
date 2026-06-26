import type { McpServerSummary } from '@bodhiapp/reference-api-types';
import type { Mcp } from '@bodhiapp/ts-client';

/**
 * Compose the read-only reference catalog with the user's own configured MCP instances. The catalog
 * carries no per-user state (by design — see mcp-techdebt.md), so "is this installed / enabled" is
 * derived client-side by joining the catalog `endpoint_url` to each instance's `mcp_server.url`.
 */

export type InstallState = 'enabled' | 'disabled' | 'none';

export interface McpJoinedRow extends McpServerSummary {
  install: InstallState;
}

/** Normalize an endpoint URL for matching: lowercase, strip a single trailing slash. */
export function normalizeEndpoint(url: string | null | undefined): string {
  if (!url) return '';
  return url.trim().toLowerCase().replace(/\/+$/, '');
}

/** Index the user's instances by normalized server URL. A disabled instance still counts as installed. */
export function indexInstances(mcps: Mcp[] | undefined): Map<string, Mcp> {
  const map = new Map<string, Mcp>();
  for (const m of mcps ?? []) {
    const key = normalizeEndpoint(m.mcp_server?.url);
    if (key) map.set(key, m);
  }
  return map;
}

/** Derive each catalog row's install state from the instance index. */
export function joinInstances(servers: McpServerSummary[], byUrl: Map<string, Mcp>): McpJoinedRow[] {
  return servers.map((s) => {
    const inst = byUrl.get(normalizeEndpoint(s.endpoint_url));
    const install: InstallState = inst ? (inst.enabled ? 'enabled' : 'disabled') : 'none';
    return { ...s, install };
  });
}

export const INSTALL_LABEL: Record<InstallState, string> = {
  enabled: 'Installed',
  disabled: 'Disabled',
  none: 'Not installed',
};
