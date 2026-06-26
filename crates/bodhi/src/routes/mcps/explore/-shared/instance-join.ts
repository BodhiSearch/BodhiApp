import type { McpServerSummary } from '@bodhiapp/reference-api-types';
import type { Mcp, McpServerResponse } from '@bodhiapp/ts-client';

/**
 * Compose the read-only reference catalog with the user's own configured MCP instances. The catalog
 * carries no per-user state (by design — see mcp-techdebt.md), so "is this installed / enabled" is
 * derived client-side by joining the catalog `endpoint_url` to each instance's `mcp_server.url`.
 *
 * The same normalized-endpoint join also resolves a catalog row to a REGISTERED server (the admin's
 * allowlist) when one exists, so the Explore rail can offer the same connect/configure actions as
 * the My MCPs rail — keyed by the registered `mcp_server_id`, not the catalog id.
 */

export type InstallState = 'enabled' | 'disabled' | 'none';

export interface McpJoinedRow extends McpServerSummary {
  install: InstallState;
  /** The matching registered server (admin allowlist), if this catalog endpoint is registered. */
  registered?: McpServerResponse;
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

/** Index the registered servers (admin allowlist) by normalized URL, for the catalog→registered join. */
export function indexRegisteredServers(servers: McpServerResponse[] | undefined): Map<string, McpServerResponse> {
  const map = new Map<string, McpServerResponse>();
  for (const s of servers ?? []) {
    const key = normalizeEndpoint(s.url);
    if (key) map.set(key, s);
  }
  return map;
}

/** Derive each catalog row's install state + registered-server match from the indexes. */
export function joinInstances(
  servers: McpServerSummary[],
  byUrl: Map<string, Mcp>,
  registeredByUrl?: Map<string, McpServerResponse>
): McpJoinedRow[] {
  return servers.map((s) => {
    const endpoint = normalizeEndpoint(s.endpoint_url);
    const inst = byUrl.get(endpoint);
    const install: InstallState = inst ? (inst.enabled ? 'enabled' : 'disabled') : 'none';
    const registered = registeredByUrl?.get(endpoint);
    return { ...s, install, registered };
  });
}

export const INSTALL_LABEL: Record<InstallState, string> = {
  enabled: 'Installed',
  disabled: 'Disabled',
  none: 'Not installed',
};
