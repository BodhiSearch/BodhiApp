import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import {
  mockCatalogModelDetail,
  mockCatalogModels,
  mockCatalogProviderDetail,
  mockCatalogProviderModels,
  mockCatalogProviders,
} from '@/test-utils/msw-v2/handlers/reference-catalog';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

import {
  buildCatalogModelsQuery,
  useCatalogModelDetail,
  useCatalogModels,
  useCatalogProviderDetail,
  useCatalogProviderModels,
  useCatalogProviders,
} from './useCatalog';

setupMswV2();

let Wrapper: ReturnType<typeof createWrapper>;

beforeEach(() => {
  Wrapper = createWrapper();
  server.use(
    ...mockAppInfoReady(),
    // A logged-in user IS present — but catalog reads use the ANONYMOUS client, so they must NOT
    // send the id_token. The onRequest captures below assert the Authorization header is absent.
    ...mockUserLoggedIn({ username: 'admin@example.com', role: 'resource_admin', id_token: 'test-id-token' })
  );
});

describe('buildCatalogModelsQuery', () => {
  it('serializes array params as repeated keys and omits empty values', () => {
    const qs = buildCatalogModelsQuery({
      q: 'claude',
      capability: ['reasoning', 'tool_call'],
      provider: ['anthropic'],
      pricing_max: 5,
      open_weights: 'open',
      page: 1,
      page_size: undefined,
    });
    const sp = new URLSearchParams(qs);
    expect(sp.getAll('capability')).toEqual(['reasoning', 'tool_call']);
    expect(sp.getAll('provider')).toEqual(['anthropic']);
    expect(sp.get('q')).toBe('claude');
    expect(sp.get('pricing_max')).toBe('5');
    expect(sp.get('open_weights')).toBe('open');
    // undefined params are omitted; page=1 kept.
    expect(sp.get('page')).toBe('1');
    expect(sp.has('page_size')).toBe(false);
    // Empty string / undefined are dropped (no stray keys).
    expect(buildCatalogModelsQuery({ q: '' })).toBe('');
  });
});

describe('useCatalog hooks — anonymous reads (no Authorization header)', () => {
  it('useCatalogModels sends array filters as repeated keys and no Bearer', async () => {
    let seen: { url: URL; authorization: string | null } | null = null;
    server.use(...mockCatalogModels({ onRequest: (info) => (seen = info) }));

    const { result } = renderHook(
      () => useCatalogModels({ capability: ['reasoning', 'vision'], provider: ['anthropic', 'openai'] }),
      { wrapper: Wrapper }
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(seen).not.toBeNull();
    expect(seen!.authorization).toBeNull();
    expect(seen!.url.searchParams.getAll('capability')).toEqual(['reasoning', 'vision']);
    expect(seen!.url.searchParams.getAll('provider')).toEqual(['anthropic', 'openai']);
    expect(result.current.data?.items.length).toBeGreaterThan(0);
  });

  it('useCatalogProviders reads anonymously', async () => {
    let seen: { url: URL; authorization: string | null } | null = null;
    server.use(...mockCatalogProviders({ onRequest: (info) => (seen = info) }));
    const { result } = renderHook(() => useCatalogProviders({ sort: 'rank' }), { wrapper: Wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(seen!.authorization).toBeNull();
    expect(result.current.data?.items.length).toBeGreaterThan(0);
  });

  it('useCatalogProviderDetail + useCatalogProviderModels are gated on slug', async () => {
    server.use(...mockCatalogProviderDetail(), ...mockCatalogProviderModels());
    const { result: gated } = renderHook(() => useCatalogProviderDetail(null), { wrapper: Wrapper });
    expect(gated.current.fetchStatus).toBe('idle');

    const { result } = renderHook(() => useCatalogProviderDetail('nano-gpt'), { wrapper: Wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.env).toContain('NANO_GPT_API_KEY');

    const { result: models } = renderHook(() => useCatalogProviderModels('nano-gpt'), { wrapper: Wrapper });
    await waitFor(() => expect(models.current.isSuccess).toBe(true));
    expect(models.current.data?.items.length).toBeGreaterThan(0);
  });

  it('useCatalogModelDetail is gated on selection and returns served_by + bridge', async () => {
    server.use(...mockCatalogModelDetail());
    const { result: gated } = renderHook(() => useCatalogModelDetail(null), { wrapper: Wrapper });
    expect(gated.current.fetchStatus).toBe('idle');

    const { result } = renderHook(() => useCatalogModelDetail({ slug: 'anthropic', modelId: 'claude-sonnet-4.5' }), {
      wrapper: Wrapper,
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.served_by.length).toBeGreaterThan(0);
    expect(result.current.data?.bridge.api_format).toBe('anthropic');
  });
});
