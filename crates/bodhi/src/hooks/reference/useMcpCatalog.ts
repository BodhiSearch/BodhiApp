import type { GetMcpServerResponse, ListMcpServersQuery, ListMcpServersResponse } from '@bodhiapp/reference-api-types';
import { keepPreviousData, useQuery } from '@tanstack/react-query';

import { REF_ENDPOINT_MCP_SERVERS, refEndpointMcpServer, referenceKeys } from './constants';
import { useAnonymousReferenceApi } from './useReferenceApi';

/**
 * Hooks for the MCP-server catalog (`/api/v1/mcp-servers`) — the Explore · MCP Servers page.
 *
 * Publicly readable, so reads go through the ANONYMOUS reference client (no id_token), same as the
 * API-model catalog. Typed by `@bodhiapp/reference-api-types`. NOT to be confused with
 * `hooks/mcps/useMcpServers` (the user's own MCP-server allowlist in BodhiApp's backend).
 */

/**
 * Build a query string from a typed MCP-servers params object, omitting empty values. The repeatable
 * `category` / `auth` params serialize as repeated keys.
 */
export function buildMcpServersQuery(params: ListMcpServersQuery): string {
  const sp = new URLSearchParams();
  const isEmpty = (v: unknown) => v === undefined || v === null || v === '';
  (Object.keys(params) as Array<keyof typeof params>).forEach((key) => {
    const value = params[key];
    if (isEmpty(value)) return;
    if (Array.isArray(value)) {
      value.forEach((v) => {
        if (!isEmpty(v)) sp.append(key, String(v));
      });
    } else {
      sp.append(key, String(value));
    }
  });
  return sp.toString();
}

/** `GET /api/v1/mcp-servers` — server list + facets, page-based. */
export function useMcpServers(params: ListMcpServersQuery) {
  const client = useAnonymousReferenceApi();
  const query = buildMcpServersQuery(params);
  return useQuery<ListMcpServersResponse>({
    queryKey: referenceKeys.mcpServers(query),
    queryFn: () => client!.get<ListMcpServersResponse>(`${REF_ENDPOINT_MCP_SERVERS}${query ? `?${query}` : ''}`),
    enabled: !!client,
    placeholderData: keepPreviousData,
  });
}

/** `GET /api/v1/mcp-servers/{id}` — full server detail. */
export function useMcpServerDetail(id: string | null) {
  const client = useAnonymousReferenceApi();
  return useQuery<GetMcpServerResponse>({
    queryKey: referenceKeys.mcpServer(id ?? ''),
    queryFn: () => client!.get<GetMcpServerResponse>(refEndpointMcpServer(id!)),
    enabled: !!client && !!id,
  });
}
