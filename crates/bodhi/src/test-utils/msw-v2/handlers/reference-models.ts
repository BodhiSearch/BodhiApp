/**
 * MSW handlers for the external Reference API (model catalog) — `https://api.getbodhi.app/`.
 *
 * Standard (non-typed) MSW `http` handlers: the reference API is NOT in BodhiApp's OpenAPI, so
 * `typedHttp` can't model it. These mirror the shipped v1 contract and let the discovery view be
 * exercised without the live API. `onRequest` exposes the request URL + Authorization header for
 * assertions (the catalog is read anonymously — no Bearer — so tests assert the header is absent).
 *
 * Base origin must match `mockAppInfo({ reference_api_url })` (default `https://api.getbodhi.app/`).
 */
import type { ListModelsResponse, Model } from '@bodhiapp/reference-api-types';

import { createDefaultCatalog, createDetailModel, createListResponse } from '@/test-fixtures/discover-models';

import { http, HttpResponse } from '../setup';

const DEFAULT_BASE = 'https://api.getbodhi.app';

type ListOptions = {
  /** Models returned (defaults to the standard catalog). */
  items?: Model[];
  /** next_cursor for the FIRST page (a second call with ?cursor returns the cursor page). */
  nextCursor?: string | null;
  /** Models returned when a ?cursor is present (the "Load more" page). */
  cursorItems?: Model[];
  base?: string;
  /** Capture the last seen request for assertions (params, auth header). */
  onRequest?: (info: { url: URL; authorization: string | null }) => void;
};

/** Stub `GET /api/v1/models`. Honors q (filters items), sort/order, cursor, limit. */
export function mockDiscoverModels(opts: ListOptions = {}) {
  const base = opts.base ?? DEFAULT_BASE;
  const items = opts.items ?? createDefaultCatalog();
  return [
    http.get(`${base}/api/v1/models`, ({ request }) => {
      const url = new URL(request.url);
      const authorization = request.headers.get('Authorization');
      opts.onRequest?.({ url, authorization });

      const q = url.searchParams.get('q');
      const cursor = url.searchParams.get('cursor');
      const sort = url.searchParams.get('sort') ?? 'downloads';
      const order = url.searchParams.get('order') ?? 'desc';

      // Cursor page (the "Load more" follow-up).
      if (cursor) {
        return HttpResponse.json<ListModelsResponse>(createListResponse(opts.cursorItems ?? [], null));
      }

      // Search filters by namespace/repo substring; setting q disables the cursor (next_cursor null).
      let result = items;
      if (q) {
        const needle = q.toLowerCase();
        result = items.filter((m) => `${m.namespace}/${m.repo}`.toLowerCase().includes(needle));
      }

      // Sort by the requested numeric column (downloads/likes/trending).
      const col = (m: Model): number =>
        sort === 'likes' ? (m.likes ?? 0) : sort === 'trending' ? (m.trending_score ?? 0) : (m.downloads ?? 0);
      result = [...result].sort((a, b) => (order === 'asc' ? col(a) - col(b) : col(b) - col(a)));

      const nextCursor = q ? null : (opts.nextCursor ?? null);
      return HttpResponse.json<ListModelsResponse>(createListResponse(result, nextCursor));
    }),
  ];
}

type DetailOptions = { model?: Model; base?: string };

/** Stub `GET /api/v1/models/{source}/{namespace}/{repo}` (single model + quants). */
export function mockDiscoverModelDetail(opts: DetailOptions = {}) {
  const base = opts.base ?? DEFAULT_BASE;
  const model = opts.model ?? createDetailModel();
  return [
    http.get(`${base}/api/v1/models/:source/:namespace/:repo`, () => {
      return HttpResponse.json<Model>(model);
    }),
  ];
}

/** Error stubs for the list endpoint (invalid token, unsupported source, etc.). */
export function mockDiscoverModelsError(
  { status = 500, error = 'internal', message }: { status?: number; error?: string; message?: string } = {},
  { base = DEFAULT_BASE }: { base?: string } = {}
) {
  return [
    http.get(`${base}/api/v1/models`, () => HttpResponse.json({ error, ...(message ? { message } : {}) }, { status })),
  ];
}
