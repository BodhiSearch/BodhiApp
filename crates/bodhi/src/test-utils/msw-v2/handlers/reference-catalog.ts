/**
 * MSW handlers for the external API-model catalog (`/api/v1/catalog/*`) — `https://api.getbodhi.app/`.
 *
 * Standard (non-typed) MSW `http` handlers: the reference API is NOT in BodhiApp's OpenAPI, so
 * `typedHttp` can't model it. These mirror the shipped catalog contract and let the Explore · API
 * Models / Providers pages be exercised without the live API. `onRequest` exposes the request URL +
 * Authorization header for assertions (the catalog is read anonymously — no Bearer — so tests
 * assert the header is absent).
 *
 * Base origin must match `mockAppInfo({ reference_api_url })` (default `https://api.getbodhi.app/`).
 */
import type {
  ModelDetailResponse,
  ModelsListResponse,
  ProviderDetailResponse,
  ProviderListResponse,
  ProviderModelsResponse,
} from '@bodhiapp/reference-api-types';

import { createModelDetail, createModelsListResponse } from '@/test-fixtures/catalog-models';
import {
  createProviderDetail,
  createProviderListResponse,
  createProviderModelsResponse,
} from '@/test-fixtures/catalog-providers';

import { http, HttpResponse } from '../setup';

const DEFAULT_BASE = 'https://api.getbodhi.app';

type RequestInfo = { url: URL; authorization: string | null };
type CommonOpts = { base?: string; onRequest?: (info: RequestInfo) => void };

function paginate<T>(items: T[], url: URL): { page: number; page_size: number; slice: T[] } {
  const page = Number(url.searchParams.get('page') ?? '1');
  const page_size = Number(url.searchParams.get('page_size') ?? '30');
  const start = (page - 1) * page_size;
  return { page, page_size, slice: items.slice(start, start + page_size) };
}

/** Stub `GET /api/v1/catalog/providers`. Honors q (filters name/slug) + page/page_size. */
export function mockCatalogProviders(opts: CommonOpts & { response?: ProviderListResponse } = {}) {
  const base = opts.base ?? DEFAULT_BASE;
  const response = opts.response ?? createProviderListResponse();
  return [
    http.get(`${base}/api/v1/catalog/providers`, ({ request }) => {
      const url = new URL(request.url);
      opts.onRequest?.({ url, authorization: request.headers.get('Authorization') });

      const q = url.searchParams.get('q')?.toLowerCase();
      let items = response.items;
      if (q) items = items.filter((p) => `${p.slug} ${p.name}`.toLowerCase().includes(q));
      const { page, page_size, slice } = paginate(items, url);
      return HttpResponse.json<ProviderListResponse>({
        ...response,
        items: slice,
        page,
        page_size,
        total: items.length,
      });
    }),
  ];
}

/** Stub `GET /api/v1/catalog/providers/{slug}`. */
export function mockCatalogProviderDetail(opts: CommonOpts & { detail?: ProviderDetailResponse } = {}) {
  const base = opts.base ?? DEFAULT_BASE;
  const detail = opts.detail ?? createProviderDetail();
  return [
    http.get(`${base}/api/v1/catalog/providers/:slug`, ({ request, params }) => {
      const url = new URL(request.url);
      opts.onRequest?.({ url, authorization: request.headers.get('Authorization') });
      return HttpResponse.json<ProviderDetailResponse>({ ...detail, slug: String(params.slug) });
    }),
  ];
}

/** Stub `GET /api/v1/catalog/providers/{slug}/models`. */
export function mockCatalogProviderModels(opts: CommonOpts & { response?: ProviderModelsResponse } = {}) {
  const base = opts.base ?? DEFAULT_BASE;
  const response = opts.response ?? createProviderModelsResponse();
  return [
    http.get(`${base}/api/v1/catalog/providers/:slug/models`, ({ request }) => {
      const url = new URL(request.url);
      opts.onRequest?.({ url, authorization: request.headers.get('Authorization') });
      return HttpResponse.json<ProviderModelsResponse>(response);
    }),
  ];
}

/** Stub `GET /api/v1/catalog/models`. Honors q (filters name/model_id) + page/page_size. */
export function mockCatalogModels(opts: CommonOpts & { response?: ModelsListResponse } = {}) {
  const base = opts.base ?? DEFAULT_BASE;
  const response = opts.response ?? createModelsListResponse();
  return [
    http.get(`${base}/api/v1/catalog/models`, ({ request }) => {
      const url = new URL(request.url);
      opts.onRequest?.({ url, authorization: request.headers.get('Authorization') });

      const q = url.searchParams.get('q')?.toLowerCase();
      let items = response.items;
      if (q) items = items.filter((m) => `${m.model_id} ${m.name}`.toLowerCase().includes(q));
      const { page, page_size, slice } = paginate(items, url);
      return HttpResponse.json<ModelsListResponse>({
        ...response,
        items: slice,
        page,
        page_size,
        total: items.length,
      });
    }),
  ];
}

/** Stub `GET /api/v1/catalog/models/{slug}/{model_id}`. */
export function mockCatalogModelDetail(opts: CommonOpts & { detail?: ModelDetailResponse } = {}) {
  const base = opts.base ?? DEFAULT_BASE;
  const detail = opts.detail ?? createModelDetail();
  return [
    http.get(`${base}/api/v1/catalog/models/:slug/:modelId`, ({ request }) => {
      const url = new URL(request.url);
      opts.onRequest?.({ url, authorization: request.headers.get('Authorization') });
      return HttpResponse.json<ModelDetailResponse>(detail);
    }),
  ];
}

/** Error stub for any catalog list endpoint. */
export function mockCatalogError(
  path: 'providers' | 'models',
  { status = 500, error = 'internal', message }: { status?: number; error?: string; message?: string } = {},
  { base = DEFAULT_BASE }: { base?: string } = {}
) {
  return [
    http.get(`${base}/api/v1/catalog/${path}`, () =>
      HttpResponse.json({ error, ...(message ? { message } : {}) }, { status })
    ),
  ];
}
