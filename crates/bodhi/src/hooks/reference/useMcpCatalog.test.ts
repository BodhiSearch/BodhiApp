import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockMcpServerDetail, mockMcpServers } from '@/test-utils/msw-v2/handlers/mcp-catalog';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

import { buildMcpServersQuery, useMcpServerDetail, useMcpServers } from './useMcpCatalog';

setupMswV2();

let Wrapper: ReturnType<typeof createWrapper>;

beforeEach(() => {
  Wrapper = createWrapper();
  server.use(
    ...mockAppInfoReady(),
    // A logged-in user IS present — but the MCP catalog is read via the ANONYMOUS client, so reads
    // must NOT send the id_token. The onRequest captures below assert Authorization is absent.
    ...mockUserLoggedIn({ username: 'admin@example.com', role: 'resource_admin', id_token: 'test-id-token' })
  );
});

describe('buildMcpServersQuery', () => {
  it('serializes repeatable category/auth as repeated keys and omits empty values', () => {
    const qs = buildMcpServersQuery({
      q: 'docs',
      category: ['Productivity', 'Search'],
      auth: ['http', 'oauth-dcr'],
      page: 1,
      page_size: undefined,
    });
    const sp = new URLSearchParams(qs);
    expect(sp.get('q')).toBe('docs');
    expect(sp.getAll('category')).toEqual(['Productivity', 'Search']);
    expect(sp.getAll('auth')).toEqual(['http', 'oauth-dcr']);
    expect(sp.get('page')).toBe('1');
    expect(sp.has('page_size')).toBe(false);
    expect(buildMcpServersQuery({ q: '' })).toBe('');
  });
});

describe('useMcpServers / useMcpServerDetail — anonymous reads (no Authorization header)', () => {
  it('useMcpServers sends no Bearer and returns items + facets', async () => {
    let seen: { url: URL; authorization: string | null } | null = null;
    server.use(...mockMcpServers({ onRequest: (info) => (seen = info) }));

    const { result } = renderHook(() => useMcpServers({ q: undefined, sort: 'name', order: 'asc' }), {
      wrapper: Wrapper,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(seen).not.toBeNull();
    expect(seen!.authorization).toBeNull();
    expect(result.current.data?.items.length).toBeGreaterThan(0);
    expect(result.current.data?.facets.auth).toContain('http');
  });

  it('useMcpServers forwards q for server-side search', async () => {
    let seen: { url: URL } | null = null;
    server.use(...mockMcpServers({ onRequest: (info) => (seen = info) }));

    const { result } = renderHook(() => useMcpServers({ q: 'notion' }), { wrapper: Wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(seen!.url.searchParams.get('q')).toBe('notion');
    expect(result.current.data?.items.every((s) => s.name.toLowerCase().includes('notion'))).toBe(true);
  });

  it('useMcpServerDetail is gated on id and returns the detail envelope', async () => {
    server.use(...mockMcpServerDetail());
    const { result: gated } = renderHook(() => useMcpServerDetail(null), { wrapper: Wrapper });
    expect(gated.current.fetchStatus).toBe('idle');

    const { result } = renderHook(() => useMcpServerDetail('notion'), { wrapper: Wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.id).toBe('notion');
    expect(result.current.data?.source).toBe('mcpservers.org');
  });
});
