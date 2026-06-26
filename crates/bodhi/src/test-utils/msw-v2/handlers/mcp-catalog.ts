/**
 * MSW handlers for the external MCP-server catalog (`/api/v1/mcp-servers`) — `https://api.getbodhi.app/`.
 *
 * Standard `http` handlers (the reference API is not in BodhiApp's OpenAPI, so `typedHttp` can't model
 * it). Mirrors the shipped contract so the Explore · MCP Servers page can be exercised without the live
 * API. The catalog is read anonymously — `onRequest` exposes the URL + Authorization header so tests can
 * assert no Bearer is sent. Base origin must match `mockAppInfo({ reference_api_url })`.
 */
import type { GetMcpServerResponse, ListMcpServersResponse } from '@bodhiapp/reference-api-types';

import { createMcpServerDetail, createMcpServersListResponse } from '@/test-fixtures/mcp-catalog';
import { http, HttpResponse } from '@/test-utils/msw-v2/setup';

const DEFAULT_BASE = 'https://api.getbodhi.app';

type RequestInfo = { url: URL; authorization: string | null };
type CommonOpts = { base?: string; onRequest?: (info: RequestInfo) => void };

function paginate<T>(items: T[], url: URL): { page: number; page_size: number; slice: T[] } {
  const page = Number(url.searchParams.get('page') ?? '1');
  const page_size = Number(url.searchParams.get('page_size') ?? '50');
  const start = (page - 1) * page_size;
  return { page, page_size, slice: items.slice(start, start + page_size) };
}

/** Stub `GET /api/v1/mcp-servers`. Honors `q` (name/description), `auth` (OR), page/page_size. */
export function mockMcpServers(opts: CommonOpts & { response?: ListMcpServersResponse } = {}) {
  const base = opts.base ?? DEFAULT_BASE;
  const response = opts.response ?? createMcpServersListResponse();
  return [
    http.get(`${base}/api/v1/mcp-servers`, ({ request }) => {
      const url = new URL(request.url);
      opts.onRequest?.({ url, authorization: request.headers.get('Authorization') });

      const q = url.searchParams.get('q')?.toLowerCase();
      const auths = url.searchParams.getAll('auth');
      let items = response.items;
      if (q) items = items.filter((s) => `${s.name} ${s.description ?? ''}`.toLowerCase().includes(q));
      if (auths.length) items = items.filter((s) => auths.includes(s.auth_type));
      const { page, page_size, slice } = paginate(items, url);
      return HttpResponse.json<ListMcpServersResponse>({
        ...response,
        items: slice,
        page,
        page_size,
        total: items.length,
      });
    }),
  ];
}

/** Stub `GET /api/v1/mcp-servers/{id}`. Returns the detail fixture; 404 for unknown ids. */
export function mockMcpServerDetail(opts: CommonOpts & { detail?: GetMcpServerResponse } = {}) {
  const base = opts.base ?? DEFAULT_BASE;
  const detail = opts.detail ?? createMcpServerDetail();
  return [
    http.get(`${base}/api/v1/mcp-servers/:id`, ({ request, params }) => {
      const url = new URL(request.url);
      opts.onRequest?.({ url, authorization: request.headers.get('Authorization') });
      if (params.id !== detail.id) {
        return HttpResponse.json({ error: 'not_found' }, { status: 404 });
      }
      return HttpResponse.json<GetMcpServerResponse>(detail);
    }),
  ];
}
